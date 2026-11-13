//! Main CLI delivery delegates to the existing preparation SDK.
use super::*;
use crate::ai_preparation::{
    State,
    setup::{self, documents},
};

fn mapped(value: crate::ValidationError) -> Error {
    error(Code::InvalidInput, &value.to_string())
}
pub(super) fn diagnose(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<setup::documents::BundleStatus>, Error> {
    let Some(_) = args.options.get("--preparation-directory") else {
        return Ok(None);
    };
    let guard = || token.is_cancelled();
    documents::status(&path(args, "--preparation-directory")?, &guard)
        .map(Some)
        .map_err(mapped)
}
pub(super) fn installation(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<crate::distribution::Observation>, Error> {
    if !args.options.contains_key("--installation-root") {
        return Ok(None);
    }
    let sha = args
        .options
        .get("--receipt-sha256")
        .and_then(|s| s.to_str())
        .ok_or_else(|| error(Code::Usage, "installation receipt digest required"))?;
    crate::distribution::status(&path(args, "--installation-root")?, sha, token).map(Some)
}
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    let guard = || token.is_cancelled();
    let value = match args.command.as_str() {
        "prepare-assets" => {
            let request = setup::Request::from_json_slice(
                &documents::read(&path(args, "--request")?, 131_072).map_err(mapped)?,
            )
            .map_err(mapped)?;
            documents::write(&request, &path(args, "--output-directory")?, &guard)
                .map_err(mapped)?
        }
        "preparation-status" => {
            documents::status(&path(args, "--preparation-directory")?, &guard).map_err(mapped)?
        }
        _ => return Ok(None),
    };
    cancelled(token)?;
    let outcome = if matches!(
        value.current.report.state,
        State::Disabled | State::PreparedBytes
    ) {
        Outcome::Complete
    } else {
        Outcome::Partial
    };
    success(&args.command, value, outcome).map(Some)
}
