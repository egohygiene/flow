//! Fixed source-entry recipe translation; native temporal acceptance stays Aniflow's.
use super::*;

fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, Error> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| error(Code::ProviderRefused, "native string field missing"))
}
fn native_path(value: &Value, field: &str) -> Result<PathBuf, Error> {
    let path = PathBuf::from(string(value, field)?);
    if !path.is_absolute() { return Err(error(Code::ProviderRefused, "native absolute locator required")); }
    Ok(path)
}
fn hash_field(value: &Value, field: &str) -> Result<String, Error> {
    let hash = string(value, field)?;
    if !release_cohort::is_digest(hash) { return Err(error(Code::ProviderRefused, "native digest field invalid")); }
    Ok(hash.into())
}
pub(super) fn validate_configuration(request: &Request) -> Result<(), Error> {
    let config: Value = serde_json::from_slice(&command::read_document(&path(&request.configuration)?, 131_072)?).map_err(|_| error(Code::InvalidInput, "explicit Aniflow recipe JSON required"))?;
    if config["schema"] != "aniflow.source-music-video-recipe/v1" || !config["extraction"].is_object() || config["recipe"]["schema"] != "aniflow.music-video-recipe/v1" {
        return Err(error(Code::InvalidInput, "only the native fixed source-entry music-video configuration is supported"));
    }
    Ok(())
}
fn invoke(context: &Context<'_>, label: &str, command: &str, args: &[OsString]) -> Result<Value, Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Envelope { schema_version: u32, command: String, status: String, result: Option<Value>, error: Option<Value> }
    let captured = context.invoke(label, args)?;
    let envelope: Envelope = serde_json::from_slice(&captured.stdout).map_err(|_| error(Code::ProviderRefused, "Aniflow native envelope is malformed"))?;
    if captured.code != Some(0) || envelope.schema_version != 1 || envelope.command != command || envelope.status != "success" || envelope.error.is_some() {
        return Err(error(Code::RecoveryRequired, "Aniflow did not accept this operation; retain native envelope and explicitly identify its run for resume"));
    }
    envelope.result.ok_or_else(|| error(Code::ProviderRefused, "Aniflow success result missing"))
}
fn base(subcommand: &str) -> Vec<OsString> { ["--output", "json", "video", subcommand].into_iter().map(OsString::from).collect() }
fn recipe_args(context: &Context<'_>) -> Result<Vec<OsString>, Error> {
    let mut args = base("music-video");
    args.extend([OsString::from("--source-video"), path(&context.plan.sources[0].path)?.into_os_string(), OsString::from("--configuration"), path(&context.plan.configuration.path)?.into_os_string()]);
    Ok(args)
}
fn validate_intent(context: &Context<'_>, intent: &Value) -> Result<String, Error> {
    let hash = hash_field(intent, "intent_sha256")?;
    let payload = &intent["payload"];
    if intent["schema"] != "aniflow.source-music-video-intent/v1" || digest(payload)? != hash
        || native_path(payload, "source_video")? != path(&context.plan.sources[0].path)?
        || native_path(payload, "provider_executable")? != path(&context.plan.provider.path)? {
        return Err(error(Code::ProviderRefused, "Aniflow intent does not bind the selected native source/provider"));
    }
    let inputs = payload["initial_inputs"].as_array().ok_or_else(|| error(Code::ProviderRefused, "Aniflow initial source identity missing"))?;
    let source = &context.plan.sources[0];
    if inputs.len() != 1 || inputs[0]["id"] != "source_video" || inputs[0]["content_sha256"] != source.sha256 || inputs[0]["byte_count"].as_u64() != Some(source.byte_count) || inputs[0]["file_count"] != 1 {
        return Err(error(Code::ProviderRefused, "Aniflow source bytes differ from approved Flow selection"));
    }
    Ok(hash)
}
fn status(context: &Context<'_>, label: &str, native_run: &Path, intent: &str) -> Result<Value, Error> {
    let mut args = base("music-video-status"); args.push(native_run.as_os_str().to_owned());
    let result = invoke(context, label, "video_music_video_status", &args)?;
    if result["schema"] != "aniflow.source-music-video-status/v1" || result["complete"] != true || string(&result, "intent_sha256")? != intent || native_path(&result, "run_directory")? != native_run {
        return Err(error(Code::RecoveryRequired, "Aniflow source recipe remains partial or its native identity changed"));
    }
    Ok(result)
}
pub(super) fn execute(context: &Context<'_>, resume: Option<&Path>) -> Result<Receipt, Error> {
    let descriptor = invoke(context, "contract", "release_contract", &["--output".into(), "json".into(), "release-contract".into()])?;
    let pinned: Value = serde_json::from_slice(&command::read_document(&path(&context.plan.descriptor.path)?, MAX_DOCUMENT)?).map_err(|_| error(Code::InvalidInput, "pinned native descriptor JSON"))?;
    if descriptor != pinned { return Err(error(Code::Unavailable, "running Aniflow contract differs from exact pinned descriptor")); }
    let mut planning = recipe_args(context)?; planning.push("--plan".into());
    let native_plan = invoke(context, "plan", "video_music_video", &planning)?;
    if native_plan["schema"] != "aniflow.source-music-video-plan/v1" { return Err(error(Code::ProviderRefused, "Aniflow native source plan schema differs")); }
    let intent = validate_intent(context, &native_plan["intent"])?;
    write(&context.evidence.join("native-plan.json"), &native_plan)?;
    let mut args = recipe_args(context)?;
    if let Some(run) = resume {
        context.require_native_run(run)?;
        // Bind to the first admitted native intent, not a newly guessed recipe.
        let original: Value = read(&context.root.join("attempt-0000/native-plan.json"))?;
        let saved: Value = read(&run.join("intent.json"))?;
        if validate_intent(context, &saved)? != intent || original["intent"] != saved {
            return Err(error(Code::RecoveryRequired, "resume native intent differs from the original approved attempt"));
        }
        args.extend(["--resume-run-directory".into(), run.as_os_str().to_owned()]);
    } else {
        args.extend(["--output-directory".into(), context.native_root().into_os_string()]);
    }
    write(&context.evidence.join("transform-started.json"), &serde_json::json!({"schema_version":"flow.native-transform-attempt/v1","plan_sha256":context.plan.plan_sha256,"native_intent_sha256":intent,"resume":resume.map(NativePath::from_path)}))?;
    let result = invoke(context, "run", "video_music_video", &args)?;
    if result["schema"] != "aniflow.source-music-video-outcome/v1" || string(&result, "intent_sha256")? != intent { return Err(error(Code::ProviderRefused, "Aniflow native outcome differs from admitted intent")); }
    let native_run = native_path(&result, "run_directory")?; context.require_native_run(&native_run)?;
    if resume.is_some_and(|expected| expected != native_run) { return Err(error(Code::ProviderRefused, "Aniflow resumed a different native run")); }
    let saved: Value = read(&native_run.join("intent.json"))?;
    if saved != native_plan["intent"] || validate_intent(context, &saved)? != intent { return Err(error(Code::ProviderRefused, "Aniflow persisted a different recipe intent")); }
    let before = status(context, "status-before", &native_run, &intent)?;
    let mut completion_args = base("completion"); completion_args.push(native_run.as_os_str().to_owned());
    let completion = invoke(context, "completion", "video_completion", &completion_args)?;
    if completion["schema"] != "aniflow.final-video-handoff/v1" || completion["publication_authority"] != "none" || completion["proof_state"] != "not_asserted" {
        return Err(error(Code::ProviderRefused, "Aniflow completion contract/authority differs"));
    }
    for field in ["completion_id", "plan_sha256", "run_manifest_sha256", "delivery_acceptance_sha256"] { hash_field(&completion, field)?; }
    let assembly = before["phases"].as_array().and_then(|phases| phases.iter().find(|p| p["phase"] == "assembly")).ok_or_else(|| error(Code::ProviderRefused, "Aniflow accepted assembly phase missing"))?;
    if assembly["plan_sha256"] != completion["plan_sha256"] { return Err(error(Code::ProviderRefused, "completion belongs to a different native assembly plan")); }
    let artifacts = completion["artifacts"].as_array().ok_or_else(|| error(Code::ProviderRefused, "native accepted final-video artifacts missing"))?;
    let mut variants = BTreeSet::new(); let mut identities = BTreeSet::new(); let mut entries = vec![];
    for artifact in artifacts {
        if !identities.insert(string(artifact, "artifact_id")?.to_owned()) || !variants.insert(string(artifact, "variant")?.to_owned()) || !["clean_archival_master", "destination_copy"].contains(&string(artifact, "variant")?) {
            return Err(error(Code::ProviderRefused, "native final-video artifact identity/variant differs"));
        }
        entries.push((native_path(artifact, "locator")?, hash_field(artifact, "sha256")?, artifact["byte_count"].as_u64().ok_or_else(|| error(Code::ProviderRefused, "native final-video byte count"))?));
    }
    if variants.len() != 2 { return Err(error(Code::ProviderRefused, "fixed source recipe requires both accepted archival master and delivery")); }
    let outputs = context.observe_artifacts(entries)?;
    let after = status(context, "status-after", &native_run, &intent)?;
    if before != after { return Err(error(Code::RecoveryRequired, "Aniflow recipe changed during completion observation")); }
    let evidence = serde_json::json!({"schema_version":"flow.aniflow-domain-acceptance/v1","native_plan":native_plan,"native_status":after,"native_completion":completion,"source_pins":context.plan.sources,"scope":"Native fixed-profile acceptance plus current confined output hashes; no creative, publication or release qualification authority"});
    context.finish(&native_run, &intent, &evidence, outputs)
}

pub(super) fn validate_retained(plan: &Plan, receipt: &Receipt, value: &Value) -> Result<(), Error> {
    let intent = &value["native_plan"]["intent"];
    let status = &value["native_status"];
    let completion = &value["native_completion"];
    if value["schema_version"] != "flow.aniflow-domain-acceptance/v1" || value["source_pins"] != serde_json::to_value(&plan.sources).map_err(|_| error(Code::InvalidInput, "source pins encoding"))?
        || intent["schema"] != "aniflow.source-music-video-intent/v1" || hash_field(intent, "intent_sha256")? != receipt.native_intent_sha256
        || digest(&intent["payload"])? != receipt.native_intent_sha256 || native_path(&intent["payload"], "source_video")? != path(&plan.sources[0].path)?
        || status["schema"] != "aniflow.source-music-video-status/v1" || status["complete"] != true || status["intent_sha256"] != receipt.native_intent_sha256
        || native_path(status, "run_directory")? != path(&receipt.native_run)? || completion["schema"] != "aniflow.final-video-handoff/v1"
        || completion["publication_authority"] != "none" || completion["proof_state"] != "not_asserted" { return Err(error(Code::RecoveryRequired, "retained Aniflow domain identity differs")); }
    let declared = completion["artifacts"].as_array().ok_or_else(|| error(Code::RecoveryRequired, "retained Aniflow completion artifacts missing"))?;
    if declared.len() != receipt.artifacts.len() { return Err(error(Code::RecoveryRequired, "retained Aniflow artifact membership changed")); }
    for (native, observed) in declared.iter().zip(&receipt.artifacts) {
        if native_path(native, "locator")? != path(&observed.path)? || native["sha256"] != observed.sha256 || native["byte_count"].as_u64() != Some(observed.byte_count) { return Err(error(Code::RecoveryRequired, "retained Aniflow accepted artifact differs")); }
    }
    let saved: Value = read(&path(&receipt.native_run)?.join("intent.json"))?;
    if saved != *intent { return Err(error(Code::RecoveryRequired, "native persisted Aniflow intent changed")); }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn paths_and_command_contracts_preserve_spaces_and_unicode() {
        let args = super::base("completion");
        assert_eq!(args.iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>(), ["--output", "json", "video", "completion"]);
        let value = serde_json::json!({"locator":"/private/native/曲 title/master.mkv"});
        assert_eq!(super::native_path(&value, "locator").unwrap(), std::path::PathBuf::from("/private/native/曲 title/master.mkv"));
        assert!(super::native_path(&serde_json::json!({"locator":"../escape"}), "locator").is_err());
    }
}
