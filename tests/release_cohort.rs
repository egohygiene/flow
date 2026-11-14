//! Authored source cases; execution is deferred with the suite acceptance work.
use flow::{NeverCancelled, release_cohort::*};

fn cohort() -> Cohort {
    Cohort::from_json_slice(include_bytes!("../examples/release-cohort-source.json")).unwrap()
}
struct Claims;
impl EvidencePort for Claims {
    fn observe(
        &self,
        row: &Row,
        _: Dimension,
        _: &dyn flow::CancellationSignal,
    ) -> Result<Observation, Error> {
        Ok(Observation::unobserved(
            row,
            "source declaration is not executed evidence",
        ))
    }
}
#[test]
fn source_inventory_keeps_unreleased_and_retained_targets_visible() {
    let inventory = cohort();
    let matrix = project(&inventory, &Claims, &NeverCancelled).unwrap();
    assert_eq!(matrix.green, 0);
    assert_eq!(matrix.rows.len(), inventory.retained_rows.len());
    assert!(inventory.targets.iter().any(|t| t == "x86_64-apple-darwin"));
    assert!(
        inventory
            .targets
            .iter()
            .any(|t| t == "x86_64-pc-windows-msvc")
    );
    let mut missing = inventory;
    missing.rows.pop();
    assert!(missing.validate().is_err());
}
struct WrongSubject;
impl EvidencePort for WrongSubject {
    fn observe(
        &self,
        _: &Row,
        _: Dimension,
        _: &dyn flow::CancellationSignal,
    ) -> Result<Observation, Error> {
        Ok(Observation {
            subject_sha256: "0".repeat(64),
            state: State::Passed,
            evidence_sha256: Some("1".repeat(64)),
            reason: "unrelated native result".into(),
        })
    }
}
#[test]
fn native_proof_for_another_row_cannot_make_a_cell_green() {
    assert!(project(&cohort(), &WrongSubject, &NeverCancelled).is_err());
}
#[test]
fn effect_and_requirement_changes_invalidate_a_row_subject() {
    let mut row = cohort().rows.remove(0);
    let original = subject(&row).unwrap();
    row.effects.push("source_mutation".into());
    assert_ne!(subject(&row).unwrap(), original);
    row.requirements.push(Requirement {
        id: "backend".into(),
        kind: "tool".into(),
        constraint: "exact pin".into(),
        preparation: "explicit offline preparation".into(),
    });
    assert_ne!(subject(&row).unwrap(), original);
}

fn expected(row: &Row) -> coverage::Expected {
    coverage::Expected {
        tool: row.tool,
        interface: row.interface,
        target: row.target.clone(),
        profile: row.profile.clone(),
        row_id: None,
    }
}
fn demand(c: &Cohort, required: Vec<coverage::Expected>) -> coverage::Request {
    coverage::Request {
        schema_version: "flow.release-coverage-request/v1".into(),
        cohort_id: c.id.clone(),
        required,
    }
}
#[test]
fn omitted_target_is_visible_even_when_the_declared_row_is_green() {
    // Synthetic trusted port only; this is never real native/release evidence.
    struct Synthetic;
    impl EvidencePort for Synthetic {
        fn observe(
            &self,
            row: &Row,
            _: Dimension,
            _: &dyn flow::CancellationSignal,
        ) -> Result<Observation, Error> {
            Ok(Observation {
                subject_sha256: subject(row)?,
                state: State::Passed,
                evidence_sha256: Some("f".repeat(64)),
                reason: "synthetic source fixture".into(),
            })
        }
    }
    let mut c = cohort();
    let row = &mut c.rows[0];
    row.implementation = Implementation::Implemented;
    row.artifact = Some(Artifact {
        version: "0.1.0".into(),
        source_revision: "a".repeat(40),
        name: "synthetic.tar.gz".into(),
        sha256: "b".repeat(64),
        byte_count: 12,
        release_url: "https://github.com/egohygiene/flow/releases/download/v0.1.0/synthetic.tar.gz"
            .into(),
    });
    row.contracts = vec![Contract {
        identifier: "flow.synthetic/v1".into(),
        sha256: "c".repeat(64),
    }];
    let first = expected(&c.rows[0]);
    let mut missing = first.clone();
    missing.target = "not-declared-target".into();
    let request = demand(&c, vec![first, missing]);
    let report = coverage::evaluate(
        project(&c, &Synthetic, &NeverCancelled).unwrap(),
        &request,
        &NeverCancelled,
    )
    .unwrap();
    assert_eq!(report.green, 1);
    assert_eq!(report.missing, 1);
    assert!(!report.all_requested_green && !report.whole_suite_qualified);
}
#[test]
fn multiple_profile_rows_require_an_explicit_row_selection() {
    let mut c = cohort();
    let mut row = c.rows[0].clone();
    row.id = "second-same-profile".into();
    c.retained_rows.push(row.id.clone());
    c.rows.push(row);
    let request = demand(&c, vec![expected(&c.rows[0])]);
    let report = coverage::evaluate(
        project(&c, &Claims, &NeverCancelled).unwrap(),
        &request,
        &NeverCancelled,
    )
    .unwrap();
    assert_eq!(report.ambiguous, 1);
    assert!(!report.all_requested_green);
    let mut selected = request;
    selected.required[0].row_id = Some(c.rows[0].id.clone());
    assert_eq!(
        coverage::evaluate(
            project(&c, &Claims, &NeverCancelled).unwrap(),
            &selected,
            &NeverCancelled
        )
        .unwrap()
        .ambiguous,
        0
    );
}
#[test]
fn duplicate_demands_and_forged_green_flags_cannot_promote_coverage() {
    let c = cohort();
    let row = expected(&c.rows[0]);
    let duplicate = demand(&c, vec![row.clone(), row.clone()]);
    assert!(
        coverage::evaluate(
            project(&c, &Claims, &NeverCancelled).unwrap(),
            &duplicate,
            &NeverCancelled
        )
        .is_err()
    );
    let mut matrix = project(&c, &Claims, &NeverCancelled).unwrap();
    matrix.rows[0].green = true;
    assert!(coverage::evaluate(matrix, &demand(&c, vec![row]), &NeverCancelled).is_err());
}
