//! Authored, unexecuted source cases for the closed delivery surface.
use flow::command::{Code, delivery::{self, Presentation}};
fn parse(args: &[&str]) -> Result<delivery::Arguments, flow::command::Error> {
    delivery::parse(args.iter().map(|value| (*value).into()).collect())
}
#[test]
fn both_candidate_acknowledgements_belong_to_approval_only() {
    let value = parse(&["approve","--plan","plan.json","--approved-by","operator","--output","approval.json","--acknowledge-trusted-unconfined","--acknowledge-unqualified-candidate"]).unwrap();
    assert!(value.acknowledge_trusted_unconfined && value.acknowledge_unqualified_candidate);
    assert!(parse(&["run","--plan","p","--approval","a","--acknowledge-unqualified-candidate"]).is_err());
    assert!(parse(&["approve","--plan","p","--approved-by","a","--output","o","--acknowledge-unqualified-candidate","--acknowledge-unqualified-candidate"]).is_err());
}
#[test]
fn native_selection_cannot_hide_source_or_provider_overrides() {
    assert!(parse(&["plan","--input","/source","--profile","aniflow-temporal","--native-selection","/selection.json"]).is_ok());
    assert!(parse(&["plan","--input","/source","--native-selection","/selection.json","--provider-executable","/other"]).is_err());
    assert!(parse(&["plan","--request","/request","--native-selection","/selection"]).is_err());
}
#[test]
fn matrix_and_resume_have_explicit_selections() {
    assert_eq!(parse(&["release-matrix"]).unwrap_err().code, Code::Usage);
    assert!(parse(&["doctor","--installations","/installed.json"]).is_err());
    assert!(parse(&["release-matrix","--cohort","/cohort.json","--format","human"]).is_ok());
    let value = parse(&["resume","--run-directory","/flow/run","--native-run-directory","/flow/run/native/retained","--format","json"]).unwrap();
    assert_eq!(value.presentation, Presentation::Json);
    assert!(parse(&["status","--run-directory","/run","--native-run-directory","/run/native/retained"]).is_err());
}
