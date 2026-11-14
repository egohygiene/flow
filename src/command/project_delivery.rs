//! Explicit preparation is distinct from read-only project preview and run.
use super::*;
use crate::project;
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    if args.command == "project-status" {
        let sha = args
            .options
            .get("--record-sha256")
            .and_then(|s| s.to_str())
            .ok_or_else(|| error(Code::Usage, "prepared project record digest required"))?;
        return success(
            "project-status",
            project::status(&path(args, "--workspace")?, sha, token)?,
            Outcome::Complete,
        )
        .map(Some);
    }
    if !["project-preview", "project-prepare"].contains(&args.command.as_str()) {
        return Ok(None);
    }
    let selected =
        project::Project::from_json_slice(&read_document(&path(args, "--project")?, 131_072)?)?;
    if args.command == "project-preview" {
        success(
            "project-preview",
            project::preview(&selected, token)?,
            Outcome::Complete,
        )
        .map(Some)
    } else {
        success(
            "project-prepare",
            project::prepare(&selected, token)?,
            Outcome::Complete,
        )
        .map(Some)
    }
}
