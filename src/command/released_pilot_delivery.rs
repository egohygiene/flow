//! Root host trust and explicit verification precede released pilot authority.
use super::*;
use crate::{pilot, release_evidence};
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    if ![
        "pilot-approve-released",
        "pilot-run-released",
        "pilot-status-released",
        "pilot-continue-released",
    ]
    .contains(&args.command.as_str())
    {
        return Ok(None);
    }
    let request = release_evidence::Request::from_json_slice(&read_document(
        &path(args, "--evidence-request")?,
        1_048_576,
    )?)?;
    let trust = release_delivery::selected_trust(args, token)?;
    let evidence = release_evidence::verify(
        &request,
        &trust,
        &path(args, "--verification-directory")?,
        args.acknowledge_verification_network,
        token,
    )?;
    if args.command == "pilot-status-released" {
        let value = pilot::released::status(&path(args, "--run-directory")?, &evidence, token)?;
        let outcome = if value.handoff.is_some() {
            Outcome::Complete
        } else {
            Outcome::RecoveryRequired
        };
        return success("pilot-status-released", value, outcome).map(Some);
    }
    let plan = pilot::Plan::from_json_slice(&read_document(&path(args, "--plan")?, 16_777_216)?)?;
    if args.command == "pilot-approve-released" {
        let by = args
            .options
            .get("--approved-by")
            .and_then(|s| s.to_str())
            .ok_or_else(|| error(Code::Usage, "explicit operator declaration required"))?
            .to_owned();
        let value = pilot::released::approve(
            &plan,
            by,
            args.acknowledge_trusted_unconfined,
            &evidence,
            token,
        )?;
        pilot::save(&plan, &path(args, "--output")?, &value)?;
        return success("pilot-approve-released", value, Outcome::Complete).map(Some);
    }
    let approval: pilot::released::Approval =
        serde_json::from_slice(&read_document(&path(args, "--approval")?, 131_072)?).map_err(
            |_| {
                error(
                    Code::InvalidInput,
                    "closed released pilot authority required",
                )
            },
        )?;
    success(
        &args.command,
        pilot::released::run(
            &plan,
            &approval,
            &evidence,
            args.command == "pilot-continue-released",
            token,
        )?,
        Outcome::Complete,
    )
    .map(Some)
}
