use serde::{Deserialize, Serialize};

use crate::{AcceptedArtifactSet, CancellationSignal, EventSink, SecretResolver};

use super::{
    DurableExecutionError, ProcessStepContext, ResumeEligibility, RunPlan, RunState, RunStepStatus,
    RunStore, StateBoundary, StateError, digest, require, schema,
};

pub const RUN_ASSESSMENT_V1: &str = "flow.run-assessment/v1";

/// One explicitly identified current context, supplied in immutable plan order.
pub struct RunStepContext<'a> {
    pub step_id: &'a str,
    pub context: ProcessStepContext<'a>,
}

/// Read-only eligibility evidence. Deserializing this report grants no authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunAssessment {
    pub schema_version: String,
    pub plan_digest: String,
    pub state_digest: String,
    pub sequence: u64,
    pub steps: Vec<RunStepAssessment>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunStepAssessment {
    pub step_id: String,
    pub recorded_status: RunStepStatus,
    pub eligibility: ResumeEligibility,
    /// A local mismatch, absent when a prerequisite prevents local assessment.
    pub stale_boundary: Option<StateBoundary>,
    /// Non-reusable direct prerequisites, in the plan's dependency order.
    pub blocked_by: Vec<String>,
}

impl RunAssessment {
    /// Validate report structure and correlation, never the freshness of its claims.
    ///
    /// # Errors
    /// Refuses unsupported schemas, wrong state identities, and contradictory
    /// ordering, dependency, status, or explanation fields.
    pub fn validate_against(&self, state: &RunState) -> Result<(), StateError> {
        schema(&self.schema_version, RUN_ASSESSMENT_V1)?;
        state.validate()?;
        require(
            self.plan_digest == state.plan_digest
                && self.state_digest == digest(state)?
                && self.sequence == state.sequence,
            "assessment state identity",
        )?;
        require(
            self.steps.len() == state.steps.len(),
            "assessment inventory",
        )?;
        for (index, step) in self.steps.iter().enumerate() {
            require(
                step.step_id == state.steps[index].step_id
                    && step.recorded_status == state.steps[index].status,
                "assessment step identity",
            )?;
            let (blocked_by, invalidated) = blockers(&state.plan, &self.steps[..index], index);
            require(
                step.blocked_by == blocked_by,
                "assessment dependency evidence",
            )?;
            let expected = if !blocked_by.is_empty() {
                require(step.stale_boundary.is_none(), "blocked local evidence")?;
                blocked_eligibility(invalidated)
            } else if let Some(boundary) = step.stale_boundary {
                require(boundary != StateBoundary::Plan, "assessment exact plan")?;
                ResumeEligibility::Invalidated
            } else {
                match step.recorded_status {
                    RunStepStatus::Pending => ResumeEligibility::Ready,
                    RunStepStatus::Succeeded => ResumeEligibility::Reusable,
                    RunStepStatus::Abandoned => ResumeEligibility::Abandoned,
                    _ => ResumeEligibility::ApprovalRequired,
                }
            };
            require(step.eligibility == expected, "assessment eligibility")?;
        }
        Ok(())
    }
}

impl RunStore {
    /// Freshly assess all steps in the exact saved plan without changing history.
    ///
    /// Non-reusable prerequisites prevent observing a descendant's future inputs.
    /// Stale evidence invalidates descendants; other unresolved prerequisites block
    /// them. Independent branches are assessed with their own current evidence.
    /// External files must remain quiescent; observations are not an atomic snapshot.
    ///
    /// # Errors
    /// Refuses corrupt history, changed plans, incomplete/reordered context inventories,
    /// and malformed contexts. Local stale evidence becomes a typed report entry.
    pub fn assess_run(
        &self,
        expected_plan: &RunPlan,
        contexts: &[RunStepContext<'_>],
    ) -> Result<RunAssessment, StateError> {
        self.check_expected_plan(expected_plan)?;
        if contexts.len() != self.state.steps.len()
            || contexts
                .iter()
                .zip(&self.state.steps)
                .any(|(current, saved)| current.step_id != saved.step_id)
        {
            return Err(StateError::ContextInventory);
        }
        let mut steps = Vec::with_capacity(contexts.len());
        for (index, current) in contexts.iter().enumerate() {
            let (blocked_by, invalidated) = blockers(&self.state.plan, &steps, index);
            let (eligibility, stale_boundary) = if blocked_by.is_empty() {
                match self.assess_local(index, &current.context) {
                    Ok(eligibility) => (eligibility, None),
                    Err(StateError::Stale { boundary }) => {
                        (ResumeEligibility::Invalidated, Some(boundary))
                    }
                    Err(error) => return Err(error),
                }
            } else {
                (blocked_eligibility(invalidated), None)
            };
            steps.push(RunStepAssessment {
                step_id: current.step_id.to_owned(),
                recorded_status: self.state.steps[index].status,
                eligibility,
                stale_boundary,
                blocked_by,
            });
        }
        Ok(RunAssessment {
            schema_version: RUN_ASSESSMENT_V1.to_owned(),
            plan_digest: self.state.plan_digest.clone(),
            state_digest: digest(&self.state)?,
            sequence: self.state.sequence,
            steps,
        })
    }

    /// Reassess the complete graph, then execute one caller-selected ready step.
    /// A saved or caller-edited assessment is never an input to this operation.
    ///
    /// # Errors
    /// Refuses any step that is not freshly ready, including completed work,
    /// stale ancestors, missing evidence, and unresolved dependencies. Execution
    /// uses the same durable intent and acceptance path as root-step `execute`.
    pub fn execute_in_plan(
        &mut self,
        expected_plan: &RunPlan,
        step_id: &str,
        contexts: &[RunStepContext<'_>],
        secrets: &dyn SecretResolver,
        cancellation: &dyn CancellationSignal,
        events: &mut dyn EventSink,
    ) -> Result<AcceptedArtifactSet, DurableExecutionError> {
        let assessment = self.assess_run(expected_plan, contexts)?;
        let index = self.state.step_index(step_id)?;
        let eligibility = assessment.steps[index].eligibility;
        if eligibility != ResumeEligibility::Ready {
            return Err(StateError::Ineligible { eligibility }.into());
        }
        self.execute_ready(
            step_id,
            &contexts[index].context,
            secrets,
            cancellation,
            events,
        )
    }
}

fn blockers(plan: &RunPlan, assessed: &[RunStepAssessment], index: usize) -> (Vec<String>, bool) {
    let mut blocked_by = Vec::new();
    let mut invalidated = false;
    for id in &plan.steps[index].depends_on {
        // Validated plans only depend on preceding steps; reports follow plan order.
        if let Some(step) = assessed.iter().find(|step| &step.step_id == id) {
            if step.eligibility != ResumeEligibility::Reusable {
                blocked_by.push(id.clone());
                invalidated |= step.eligibility == ResumeEligibility::Invalidated;
            }
        }
    }
    (blocked_by, invalidated)
}

const fn blocked_eligibility(invalidated: bool) -> ResumeEligibility {
    if invalidated {
        ResumeEligibility::Invalidated
    } else {
        ResumeEligibility::DependencyBlocked
    }
}
