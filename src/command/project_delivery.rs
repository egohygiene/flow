//! Explicit preparation is distinct from read-only project preview and run.
use super::*;
use crate::project;
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    if args.command=="project-resolve"{
        let preset_path=path(args,"--preset")?;let selection_path=path(args,"--selection")?;
        let preset_bytes=read_document(&preset_path,131_072)?;let selection_bytes=read_document(&selection_path,131_072)?;
        let preset=project::preset::Preset::from_json_slice(&preset_bytes)?;let selection=project::preset::Selection::from_json_slice(&selection_bytes)?;
        let resolution=project::preset::resolve(&preset,&selection)?;
        if read_document(&preset_path,131_072)?!=preset_bytes||read_document(&selection_path,131_072)?!=selection_bytes{return Err(error(Code::StalePlan,"preset or project selection changed during resolution"));}
        cancelled(token)?;let output=path(args,"--output")?;write_document(&output,&resolution.project)?;
        let file=crate::native::FilePin::observe(&NativePath::from_path(&output),131_072,token)?;
        return success(&args.command,serde_json::json!({"schema_version":"flow.publishing-project-resolution-command/v1","resolution":resolution,"project_file":file}),Outcome::Complete).map(Some);
    }
    if args.command=="project-status"{let sha=args.options.get("--record-sha256").and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"prepared project record digest required"))?;return success("project-status",project::status(&path(args,"--workspace")?,sha,token)?,Outcome::Complete).map(Some);}
    if !["project-preview","project-prepare"].contains(&args.command.as_str()){return Ok(None);}
    let selected=project::Project::from_json_slice(&read_document(&path(args,"--project")?,131_072)?)?;
    if args.command=="project-preview"{success("project-preview",project::preview(&selected,token)?,Outcome::Complete).map(Some)}
    else{success("project-prepare",project::prepare(&selected,token)?,Outcome::Complete).map(Some)}
}
