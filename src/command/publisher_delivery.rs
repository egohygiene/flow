//! Thin clean-install delivery over opaque publisher verification and copying.
use super::*;
use crate::{distribution, release_evidence::publisher};
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    if !["publisher-verify", "suite-install-released"].contains(&args.command.as_str()) {
        return Ok(None);
    }
    let request = publisher::Request::from_json_slice(&read_document(
        &path(args, "--publisher-request")?,
        1_048_576,
    )?)?;
    let trust = release_delivery::selected_trust(args, token)?;
    let verified = publisher::verify(
        &request,
        &trust,
        &path(args, "--verification-directory")?,
        args.acknowledge_verification_network,
        token,
    )?;
    if args.command == "publisher-verify" {
        return success("publisher-verify", verified.summary(), Outcome::Complete).map(Some);
    }
    let installation = if args.options.contains_key("--request") {
        distribution::Request::from_json_slice(&read_document(
            &path(args, "--request")?,
            1_048_576,
        )?)?
    } else {
        let sha = args
            .options
            .get("--bundle-sha256")
            .and_then(|s| s.to_str())
            .ok_or_else(|| error(Code::Usage, "exact offline bundle receipt digest required"))?;
        distribution::open_bundle(
            &path(args, "--bundle")?,
            sha,
            &path(args, "--install-directory")?,
            token,
        )?
    };
    success(
        "suite-install-released",
        distribution::install_authenticated(&installation, &verified, token)?,
        Outcome::Complete,
    )
    .map(Some)
}
