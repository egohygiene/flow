//! Authored source cases; execution is deferred with the suite acceptance work.
use flow::{NeverCancelled, release_cohort::*};

fn cohort() -> Cohort { Cohort::from_json_slice(include_bytes!("../examples/release-cohort-source.json")).unwrap() }
struct Claims;
impl EvidencePort for Claims {
    fn observe(&self, row: &Row, _: Dimension, _: &dyn flow::CancellationSignal) -> Result<Observation, Error> {
        Ok(Observation::unobserved(row, "source declaration is not executed evidence"))
    }
}
#[test]
fn source_inventory_keeps_unreleased_and_retained_targets_visible() {
    let inventory = cohort();
    let matrix = project(&inventory, &Claims, &NeverCancelled).unwrap();
    assert_eq!(matrix.green, 0);
    assert_eq!(matrix.rows.len(), inventory.retained_rows.len());
    assert!(inventory.targets.iter().any(|t| t == "x86_64-apple-darwin"));
    assert!(inventory.targets.iter().any(|t| t == "x86_64-pc-windows-msvc"));
    let mut missing = inventory;
    missing.rows.pop();
    assert!(missing.validate().is_err());
}
struct WrongSubject;
impl EvidencePort for WrongSubject {
    fn observe(&self, _: &Row, _: Dimension, _: &dyn flow::CancellationSignal) -> Result<Observation, Error> {
        Ok(Observation { subject_sha256: "0".repeat(64), state: State::Passed, evidence_sha256: Some("1".repeat(64)), reason: "unrelated native result".into() })
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
    row.requirements.push(Requirement { id: "backend".into(), kind: "tool".into(), constraint: "exact pin".into(), preparation: "explicit offline preparation".into() });
    assert_ne!(subject(&row).unwrap(), original);
}
