use super::*;
use crate::static_project;
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    match args.command.as_str() {
        "publication-project-preview" | "publication-project-prepare" => {
            let project = static_project::Project::from_json_slice(&read_document(
                &path(args, "--project")?,
                1_048_576,
            )?)?;
            if args.command == "publication-project-preview" {
                success(
                    &args.command,
                    static_project::preview(&project, token)?,
                    Outcome::Complete,
                )
                .map(Some)
            } else {
                success(
                    &args.command,
                    static_project::prepare(&project, token)?,
                    Outcome::Complete,
                )
                .map(Some)
            }
        }
        "publication-project-status" => {
            let sha = args
                .options
                .get("--record-sha256")
                .and_then(|s| s.to_str())
                .ok_or_else(|| error(Code::Usage, "exact static project record digest required"))?;
            success(
                &args.command,
                static_project::status(&path(args, "--workspace")?, sha, token)?,
                Outcome::Partial,
            )
            .map(Some)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_preparation_cannot_approve_or_execute() {
        let argv = |s: &[&str]| -> Vec<OsString> { s.iter().map(|v| OsString::from(*v)).collect() };
        assert!(
            parse(argv(&[
                "publication-project-prepare",
                "--project",
                "private 作品.json"
            ]))
            .is_ok()
        );
        assert!(
            parse(argv(&[
                "publication-project-prepare",
                "--project",
                "p",
                "--approved-by",
                "operator"
            ]))
            .is_err()
        );
        assert!(parse(argv(&["publication-project-status", "--workspace", "w"])).is_err());
    }
}
