use super::*;
use crate::delivery::publishing_readiness;
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    if args.command != "publishing-readiness" {
        return Ok(None);
    }
    let request = publishing_readiness::Request::from_json_slice(&read_document(
        &path(args, "--request")?,
        131_072,
    )?)?;
    let value = if args.options.contains_key("--output") {
        publishing_readiness::save(&request, &path(args, "--output")?, token)?
    } else {
        publishing_readiness::report(&request, token)?
    };
    // The report deliberately leaves current owner/qualification/platform work
    // explicit; local snapshot readback cannot declare publishing all-green.
    success(&args.command, value, Outcome::Partial).map(Some)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_has_no_execution_or_approval_surface() {
        let argv = |items: &[&str]| -> Vec<OsString> {
            items.iter().map(|v| OsString::from(*v)).collect()
        };
        assert!(
            parse(argv(&[
                "publishing-readiness",
                "--request",
                "private 音/report-request.json",
                "--format",
                "json"
            ]))
            .is_ok()
        );
        assert!(
            parse(argv(&[
                "publishing-readiness",
                "--request",
                "r",
                "--approved-by",
                "operator"
            ]))
            .is_err()
        );
        assert!(
            parse(argv(&[
                "publishing-readiness",
                "--request",
                "r",
                "--acknowledge-verification-network"
            ]))
            .is_err()
        );
        assert!(parse(argv(&["publishing-readiness", "--output", "report.json"])).is_err());
    }
}
