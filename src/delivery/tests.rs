//! Authored review-scope regressions; not executed or native proof.
use super::*;
fn declaration(scope:ReviewScope,decision:Decision)->Review{Review{schema_version:"flow.delivery-review/v1".into(),package_subject_sha256:"a".repeat(64),scope,decision,reviewed_by:"synthetic test declaration".into(),reviewed_at:"not a real review".into(),notes:"unit fixture only".into()}}
#[test]
fn no_supplied_review_stays_pending_without_identity_or_publish_authority(){
    let summary=reviews(&[],&"a".repeat(64)).unwrap();assert_eq!(summary.pending.len(),5);assert_eq!(summary.state,"review_pending");assert!(!summary.identity_authenticated);
}
#[test]
fn stale_or_duplicate_scope_cannot_satisfy_package_review(){
    let a=declaration(ReviewScope::Words,Decision::Accept);
    assert!(reviews(&[a.clone()],&"b".repeat(64)).is_err());
    assert!(reviews(&[a.clone(),a],&"a".repeat(64)).is_err());
}
#[test]
fn a_rejection_remains_visible_with_other_accept_declarations(){
    let values=SCOPES.into_iter().map(|s|declaration(s,if s==ReviewScope::FinalPackage{Decision::Reject}else{Decision::Accept})).collect::<Vec<_>>();
    let summary=reviews(&values,&"a".repeat(64)).unwrap();assert_eq!(summary.state,"changes_requested");assert_eq!(summary.changes_requested,vec![ReviewScope::FinalPackage]);assert!(!summary.identity_authenticated);
}
