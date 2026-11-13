//! Canonical single-file / ordered-collection publication translation.
//! Renderflow retains conversion, validation, fidelity and graph recovery rules.
use super::*;
use std::collections::BTreeMap;
fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, Error> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| {
        error(
            Code::ProviderRefused,
            "native Renderflow string field missing",
        )
    })
}
fn forest_hash(plan: &Value) -> Result<String, Error> {
    let identity = string(&plan["artifact_forest"], "identity")?
        .strip_prefix("sha256:")
        .ok_or_else(|| error(Code::ProviderRefused, "native forest identity algorithm"))?;
    if !release_cohort::is_digest(identity) {
        return Err(error(
            Code::ProviderRefused,
            "native forest identity digest",
        ));
    }
    Ok(identity.into())
}
fn output_root(request: &Request) -> Result<PathBuf, Error> {
    Ok(path(&request.run_directory)?.join("native/output"))
}
fn config(request: &Request) -> Result<Value, Error> {
    serde_json::from_slice(&command::read_document(
        &path(&request.configuration)?,
        1_048_576,
    )?)
    .map_err(|_| {
        error(
            Code::InvalidInput,
            "explicit Renderflow v2 JSON configuration required",
        )
    })
}
fn selected_sources(value: &Value) -> Result<Vec<&Value>, Error> {
    let sources = value["sources"].as_array().ok_or_else(|| {
        error(
            Code::InvalidInput,
            "explicit native source declarations required",
        )
    })?;
    if sources.is_empty() || sources.len() > 8193 {
        return Err(error(Code::InvalidInput, "native source declaration bound"));
    }
    let mut artifacts = BTreeMap::new();
    let mut collection = None;
    for source in sources {
        let id = string(source, "id")?;
        if id.is_empty()
            || source.get("uri").is_some_and(|v| !v.is_null())
            || source.get("immutable").is_some_and(|v| v != true)
        {
            return Err(error(
                Code::InvalidInput,
                "only explicit immutable local native sources are supported",
            ));
        }
        match source
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("artifact")
        {
            "artifact" => {
                if artifacts.insert(id, source).is_some() {
                    return Err(error(
                        Code::InvalidInput,
                        "duplicate native source identity",
                    ));
                }
            }
            "collection" if collection.is_none() => collection = Some(source),
            _ => {
                return Err(error(
                    Code::Unavailable,
                    "this Flow profile supports one file or one explicit ordered collection; scoped native packages use their own future route",
                ));
            }
        }
    }
    let selected = if let Some(collection) = collection {
        let members = collection["members"]
            .as_array()
            .ok_or_else(|| error(Code::InvalidInput, "ordered member source IDs required"))?;
        let mut selected = vec![];
        let mut seen = BTreeSet::new();
        for member in members {
            let id = member
                .as_str()
                .ok_or_else(|| error(Code::InvalidInput, "native collection source ID"))?;
            if !seen.insert(id) {
                return Err(error(
                    Code::InvalidInput,
                    "duplicate native collection source ID",
                ));
            }
            selected.push(
                *artifacts
                    .get(id)
                    .ok_or_else(|| error(Code::InvalidInput, "collection source ID missing"))?,
            );
        }
        if selected.len() != artifacts.len() || artifacts.contains_key(string(collection, "id")?) {
            return Err(error(
                Code::InvalidInput,
                "unselected or ambiguous native source declarations",
            ));
        }
        selected
    } else if artifacts.len() == 1 {
        artifacts.into_values().collect()
    } else {
        return Err(error(
            Code::InvalidInput,
            "multiple files require an explicit ordered native collection",
        ));
    };
    Ok(selected)
}
pub(super) fn validate_configuration(request: &Request) -> Result<(), Error> {
    let value = config(request)?;
    if value["schema"] != "renderflow/v2"
        || value["output"]["bundle_root"].as_str().map(Path::new)
            != Some(output_root(request)?.as_path())
        || value["output"]
            .get("collision")
            .is_some_and(|v| v != "error")
        || value.get("transforms").is_some_and(|v| !v.is_null())
        || value["execution"]
            .get("network")
            .is_some_and(|v| v != "deny")
        || value["execution"].get("ai").is_some_and(|v| v != "deny")
        || value["execution"]["requirements"]["offline"] != true
        || value["execution"]["requirements"]["local_only"] != true
    {
        return Err(error(
            Code::InvalidInput,
            "native canonical candidate requires explicit disjoint output, error collisions, offline/local-only, no network/AI or external transforms",
        ));
    }
    let sources = selected_sources(&value)?;
    if sources.len() != request.inputs.len() {
        return Err(error(
            Code::InvalidInput,
            "native source scope differs from explicit selection",
        ));
    }
    for (source, selected) in sources.iter().zip(&request.inputs) {
        if string(source, "path").map(Path::new)? != path(selected)? {
            return Err(error(
                Code::InvalidInput,
                "native ordered source path differs from selected input",
            ));
        }
    }
    Ok(())
}
fn invoke(context: &Context<'_>, label: &str, args: &[OsString]) -> Result<Value, Error> {
    let captured = context.invoke(label, args)?;
    if captured.code != Some(0) {
        return Err(error(
            Code::RecoveryRequired,
            "Renderflow native command did not complete; retain attempt and native receipt",
        ));
    }
    serde_json::from_slice(&captured.stdout).map_err(|_| {
        error(
            Code::ProviderRefused,
            "Renderflow structured result missing or malformed",
        )
    })
}
fn validate_plan(context: &Context<'_>, native: &Value) -> Result<String, Error> {
    let forest = forest_hash(native)?;
    if native["source_collection"].is_object() {
        let members = native["source_collection"]["members"]
            .as_array()
            .ok_or_else(|| {
                error(
                    Code::ProviderRefused,
                    "native ordered collection identity missing",
                )
            })?;
        if members.len() != context.plan.sources.len() {
            return Err(error(
                Code::ProviderRefused,
                "native collection scope changed",
            ));
        }
        for (member, pin) in members.iter().zip(&context.plan.sources) {
            if string(member, "locator").map(Path::new)? != path(&pin.path)?
                || member["artifact"]["digest"] != format!("sha256:{}", pin.sha256)
                || member["artifact"]["size_bytes"].as_u64() != Some(pin.byte_count)
            {
                return Err(error(
                    Code::ProviderRefused,
                    "native ordered collection bytes/order changed",
                ));
            }
        }
    } else if context.plan.sources.len() != 1
        || native["source_artifact"]["digest"]
            != format!("sha256:{}", context.plan.sources[0].sha256)
        || native["source_artifact"]["size_bytes"].as_u64()
            != Some(context.plan.sources[0].byte_count)
    {
        return Err(error(
            Code::ProviderRefused,
            "native file source identity differs",
        ));
    }
    let branches = native["artifact_forest"]["branches"]
        .as_array()
        .ok_or_else(|| {
            error(
                Code::ProviderRefused,
                "native selected target inventory missing",
            )
        })?;
    if !branches.iter().any(|branch| branch["state"] == "selected")
        || branches.iter().any(|branch| {
            ["unsupported", "unavailable", "budget_pruned"]
                .iter()
                .any(|state| branch["state"] == *state)
        })
    {
        return Err(error(
            Code::Unavailable,
            "native route has no complete available target selection; missing branches stay visible",
        ));
    }
    Ok(forest)
}
fn verify_manifest(
    context: &Context<'_>,
    native_plan: &Value,
    bytes: &[u8],
) -> Result<Value, Error> {
    let manifest: Value = serde_json::from_slice(bytes)
        .map_err(|_| error(Code::ProviderRefused, "native Renderflow receipt JSON"))?;
    let artifact_manifest = &manifest["artifact_manifest"];
    if manifest["schema_version"] != "renderflow.run/v1"
        || manifest["engine_version"]
            != context
                .plan
                .row
                .artifact
                .as_ref()
                .ok_or_else(|| error(Code::InvalidInput, "native artifact pin missing"))?
                .version
        || manifest["state"] != "complete"
        || artifact_manifest["schema_version"] != "renderflow.artifact-manifest/v1"
        || string(artifact_manifest, "output_dir").map(Path::new)?
            != output_root(&context.plan.request)?
        || manifest["artifact_forest"]["identity"] != native_plan["artifact_forest"]["identity"]
    {
        return Err(error(
            Code::RecoveryRequired,
            "native run is partial, stale or belongs to a different admitted forest/output",
        ));
    }
    let artifacts = artifact_manifest["artifacts"].as_array().ok_or_else(|| {
        error(
            Code::ProviderRefused,
            "native source/artifact inventory missing",
        )
    })?;
    for source in &context.plan.sources {
        if !artifacts.iter().any(|a| {
            a["lifecycle"] == "source"
                && a["digest"]["algorithm"] == "sha256"
                && a["digest"]["value"] == source.sha256
                && a["size_bytes"].as_u64() == Some(source.byte_count)
        }) {
            return Err(error(
                Code::ProviderRefused,
                "native run does not retain selected source byte identity",
            ));
        }
    }
    Ok(manifest)
}
fn entries(
    report: &Value,
    native_plan: &Value,
    root: &Path,
    manifest_sha256: &str,
) -> Result<Vec<(PathBuf, String, u64)>, Error> {
    if report["schema_version"] != "renderflow.publication-verification/v1"
        || report["run_state"] != "complete"
        || report["receipt_sha256"] != manifest_sha256
        || report["has_publications"] != true
    {
        return Err(error(
            Code::ProviderRefused,
            "native publication verifier did not accept exact complete receipt",
        ));
    }
    let verified = report["verified_publications"]
        .as_array()
        .ok_or_else(|| error(Code::ProviderRefused, "native actual publications missing"))?;
    if report["verified_publication_count"].as_u64() != Some(verified.len() as u64) {
        return Err(error(
            Code::ProviderRefused,
            "native publication count differs",
        ));
    }
    let branches = native_plan["artifact_forest"]["branches"]
        .as_array()
        .ok_or_else(|| error(Code::ProviderRefused, "native target branch inventory"))?;
    let selected: BTreeSet<_> = branches
        .iter()
        .filter(|b| b["state"] == "selected")
        .map(|b| string(b, "target_id"))
        .collect::<Result<_, _>>()?;
    let mut seen = BTreeSet::new();
    let mut outputs = vec![];
    for artifact in verified {
        let target = string(artifact, "target_id")?;
        if !selected.contains(target)
            || !seen.insert(target)
            || !["valid", "valid_with_warnings"].contains(&string(artifact, "validation")?)
        {
            return Err(error(
                Code::ProviderRefused,
                "unselected, duplicate or domain-unaccepted native publication",
            ));
        }
        let relative = string(artifact, "locator")?
            .strip_prefix("bundle:")
            .ok_or_else(|| {
                error(
                    Code::ProviderRefused,
                    "native publication locator must be bundle-relative",
                )
            })?;
        let relative = Path::new(relative);
        if relative.as_os_str().is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(error(
                Code::ProviderRefused,
                "native publication locator escaped output",
            ));
        }
        let sha = string(artifact, "sha256")?.to_owned();
        outputs.push((
            root.join(relative),
            sha,
            artifact["size_bytes"].as_u64().ok_or_else(|| {
                error(
                    Code::ProviderRefused,
                    "native publication byte size missing",
                )
            })?,
        ));
    }
    if selected != seen || selected.is_empty() {
        return Err(error(
            Code::RecoveryRequired,
            "not every selected native target has accepted publication evidence",
        ));
    }
    Ok(outputs)
}
pub(super) fn execute(context: &Context<'_>, resume: Option<&Path>) -> Result<Receipt, Error> {
    validate_configuration(&context.plan.request)?;
    let descriptor = invoke(context, "contract", &["release-contract".into()])?;
    let pinned: Value = read(&path(&context.plan.descriptor.path)?)?;
    if descriptor != pinned {
        return Err(error(
            Code::Unavailable,
            "running Renderflow descriptor differs from pinned contract bytes",
        ));
    }
    let root = output_root(&context.plan.request)?;
    if let Some(selected) = resume {
        context.require_native_run(selected)?;
        if selected != root {
            return Err(error(
                Code::RecoveryRequired,
                "resume must name the originally selected native output root",
            ));
        }
    } else if root
        .try_exists()
        .map_err(|_| error(Code::Io, "native output state unavailable"))?
    {
        return Err(error(
            Code::RecoveryRequired,
            "fresh native output required; preserve existing output",
        ));
    }
    let build = vec![
        OsString::from("build"),
        "--config".into(),
        path(&context.plan.configuration.path)?.into_os_string(),
    ];
    let mut planning = build.clone();
    planning.push("--dry-run".into());
    let native_plan = invoke(context, "plan", &planning)?;
    let intent = validate_plan(context, &native_plan)?;
    if resume.is_some() {
        let original: Value = read(&context.root.join("attempt-0000/native-plan.json"))?;
        if original["artifact_forest"]["identity"] != native_plan["artifact_forest"]["identity"] {
            return Err(error(
                Code::RecoveryRequired,
                "native graph resume would change the original admitted forest",
            ));
        }
    }
    write(&context.evidence.join("native-plan.json"), &native_plan)?;
    write(
        &context.evidence.join("transform-started.json"),
        &serde_json::json!({"schema_version":"flow.native-transform-attempt/v1","plan_sha256":context.plan.plan_sha256,"native_intent_sha256":intent,"resume":resume.map(NativePath::from_path)}),
    )?;
    let mut build = build;
    if resume.is_some() {
        build.push("--resume".into());
    }
    let captured = context.invoke("run", &build)?;
    if captured.code != Some(0) {
        return Err(error(
            Code::RecoveryRequired,
            "native Renderflow build failed/partial; inspect receipt before explicit graph resume",
        ));
    }
    context.require_native_run(&root)?;
    let manifest_path = root.join("renderflow-run.json");
    let manifest_bytes = command::read_document(&manifest_path, MAX_DOCUMENT)?;
    let manifest = verify_manifest(context, &native_plan, &manifest_bytes)?;
    let manifest_sha256 = release_cohort::sha256(&manifest_bytes);
    let report = invoke(
        context,
        "publication-status",
        &[
            "publication-status".into(),
            "--manifest".into(),
            manifest_path.as_os_str().to_owned(),
            "--manifest-sha256".into(),
            manifest_sha256.clone().into(),
            "--output-root".into(),
            root.as_os_str().to_owned(),
        ],
    )?;
    let outputs =
        context.observe_artifacts(entries(&report, &native_plan, &root, &manifest_sha256)?)?;
    if command::read_document(&manifest_path, MAX_DOCUMENT)? != manifest_bytes {
        return Err(error(
            Code::RecoveryRequired,
            "native receipt changed during accepted payload observation",
        ));
    }
    let evidence = serde_json::json!({"schema_version":"flow.renderflow-domain-acceptance/v1","native_plan":native_plan,"native_manifest":manifest,"native_manifest_sha256":manifest_sha256,"native_publications":report,"source_pins":context.plan.sources,"scope":"Native domain validation and current confined publication hashes; original fidelity retained; no creative/publishing/release authority"});
    context.finish(&root, &intent, &evidence, outputs)
}
pub(super) fn validate_retained(
    plan: &Plan,
    receipt: &Receipt,
    value: &Value,
) -> Result<(), Error> {
    let root = output_root(&plan.request)?;
    if value["schema_version"] != "flow.renderflow-domain-acceptance/v1"
        || forest_hash(&value["native_plan"])? != receipt.native_intent_sha256
        || path(&receipt.native_run)? != root
        || value["native_manifest"]["state"] != "complete"
        || value["source_pins"]
            != serde_json::to_value(&plan.sources)
                .map_err(|_| error(Code::InvalidInput, "source pins encoding"))?
    {
        return Err(error(
            Code::RecoveryRequired,
            "retained Renderflow native domain identity differs",
        ));
    }
    let bytes = command::read_document(&root.join("renderflow-run.json"), MAX_DOCUMENT)?;
    let manifest_sha = release_cohort::sha256(&bytes);
    let manifest: Value = serde_json::from_slice(&bytes).map_err(|_| {
        error(
            Code::RecoveryRequired,
            "retained Renderflow receipt malformed",
        )
    })?;
    if manifest != value["native_manifest"] || value["native_manifest_sha256"] != manifest_sha {
        return Err(error(
            Code::RecoveryRequired,
            "native Renderflow receipt changed",
        ));
    }
    let entries = entries(
        &value["native_publications"],
        &value["native_plan"],
        &root,
        &manifest_sha,
    )?;
    if entries.len() != receipt.artifacts.len() {
        return Err(error(
            Code::RecoveryRequired,
            "retained Renderflow publication membership differs",
        ));
    }
    for ((file, sha, count), observed) in entries.into_iter().zip(&receipt.artifacts) {
        if file != path(&observed.path)? || sha != observed.sha256 || count != observed.byte_count {
            return Err(error(
                Code::RecoveryRequired,
                "retained Renderflow accepted artifact differs",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    #[test]
    fn collection_order_is_semantic_and_unselected_members_are_refused() {
        let value = json!({"sources":[{"id":"second","path":"/pages/2.png"},{"id":"first","path":"/pages/1.png"},{"id":"book","kind":"collection","members":["first","second"]}]});
        let selected = super::selected_sources(&value).unwrap();
        assert_eq!(selected[0]["id"], "first");
        assert_eq!(selected[1]["id"], "second");
        let mut omitted = value;
        omitted["sources"][2]["members"] = json!(["first"]);
        assert!(super::selected_sources(&omitted).is_err());
    }
    #[test]
    fn a_partial_run_with_publications_cannot_be_domain_acceptance() {
        assert!(super::entries(&json!({"schema_version":"renderflow.publication-verification/v1","run_state":"partial","receipt_sha256":"a".repeat(64),"has_publications":true}), &json!({}), std::path::Path::new("/native/output"), &"a".repeat(64)).is_err());
    }
}
