//! Thin product delivery over native and cohort library owners.
use super::*;
use crate::{
    native as provider,
    release_cohort::{self, Cohort, InstalledFiles, LocalFiles, Matrix},
};
use std::collections::BTreeSet;
fn schema(bytes: &[u8]) -> Result<String, Error> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| error(Code::InvalidInput, "versioned JSON document required"))?;
    value["schema_version"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            error(
                Code::InvalidInput,
                "explicit document schema_version required",
            )
        })
}
fn cohort_error(value: release_cohort::Error) -> Error {
    match value {
        release_cohort::Error::Cancelled => error(Code::Cancelled, "cohort observation cancelled"),
        _ => error(
            Code::InvalidInput,
            "cohort or selected installation evidence could not be observed",
        ),
    }
}
fn matrix(args: &Arguments, token: &dyn CancellationSignal) -> Result<Matrix, Error> {
    let cohort = Cohort::from_json_slice(&read_document(&path(args, "--cohort")?, 1_048_576)?)
        .map_err(cohort_error)?;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ContractFile {
        identifier: String,
        path: NativePath,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Installation {
        row_id: String,
        executable: NativePath,
        contract_files: Vec<ContractFile>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Selection {
        schema_version: String,
        installations: Vec<Installation>,
    }
    let mut installations = vec![];
    if args.options.contains_key("--installations") {
        let selection: Selection =
            serde_json::from_slice(&read_document(&path(args, "--installations")?, 1_048_576)?)
                .map_err(|_| {
                    error(
                        Code::InvalidInput,
                        "closed installed cohort file selection required",
                    )
                })?;
        if selection.schema_version != "flow.installed-cohort/v1"
            || selection.installations.len() > cohort.rows.len()
        {
            return Err(error(
                Code::InvalidInput,
                "installed cohort selection identity/bounds",
            ));
        }
        let mut rows = BTreeSet::new();
        for selected in selection.installations {
            if !rows.insert(selected.row_id.clone())
                || !cohort.rows.iter().any(|r| r.id == selected.row_id)
                || selected.contract_files.len() > 64
            {
                return Err(error(
                    Code::InvalidInput,
                    "unknown/duplicate installed row or contract bound",
                ));
            }
            let mut contracts = BTreeSet::new();
            let mut files = vec![];
            for file in selected.contract_files {
                if !contracts.insert(file.identifier.clone()) {
                    return Err(error(
                        Code::InvalidInput,
                        "duplicate installed contract identity",
                    ));
                }
                files.push((file.identifier, provider::path(&file.path)?));
            }
            installations.push(InstalledFiles {
                row_id: selected.row_id,
                executable: provider::path(&selected.executable)?,
                contract_files: files,
            });
        }
    }
    release_cohort::project(&cohort, &LocalFiles { installations }, token).map_err(cohort_error)
}
fn native_plan(
    args: &Arguments,
    request: &provider::Request,
    token: &dyn CancellationSignal,
) -> Result<Envelope, Error> {
    let plan = provider::prepare(request, token)?;
    if args.options.contains_key("--output") {
        provider::save_document(&plan, &path(args, "--output")?, &plan)?;
    }
    success(&args.command, plan, Outcome::Complete)
}
fn local_native(
    args: &Arguments,
    request: &crate::local_input::Request,
    selection: &provider::Request,
    token: &dyn CancellationSignal,
) -> Result<Envelope, Error> {
    if args.command == "inspect" {
        return success(
            "inspect",
            crate::local_input::inspect(request, token)?,
            Outcome::Complete,
        );
    }
    let value = crate::local_input::plan_native(request, selection, token)?;
    if args.options.contains_key("--output") {
        provider::save_document(
            &value.command_plan,
            &path(args, "--output")?,
            &value.command_plan,
        )?;
    }
    success("plan", value, Outcome::Complete)
}
/// `None` leaves the original released Optiflow delivery path in charge.
pub(super) fn execute(
    args: &Arguments,
    token: &dyn CancellationSignal,
) -> Result<Option<Envelope>, Error> {
    match args.command.as_str() {
        "release-matrix" => {
            let value = matrix(args, token)?;
            release_delivery::matrix_response(args, value, token).map(Some)
        }
        "doctor" => {
            let executable = args
                .options
                .get("--provider-executable")
                .map(|_| path(args, "--provider-executable"))
                .transpose()?;
            let released = doctor(executable.as_deref(), token)?;
            let projection = if args.options.contains_key("--cohort") {
                Some(matrix(args, token)?)
            } else {
                None
            };
            let preparation = preparation_delivery::diagnose(args, token)?;
            let installation = preparation_delivery::installation(args, token)?;
            let outcome = if projection.as_ref().is_some_and(|m| m.green == m.rows.len()) {
                Outcome::Complete
            } else if released.selected_provider.is_some()
                || projection.is_some()
                || preparation.is_some()
                || installation.is_some()
            {
                Outcome::Partial
            } else {
                Outcome::Unavailable
            };
            let value = serde_json::json!({"schema_version":"flow.command-doctor/v2","released_optiflow":released,
                "source_candidate_routes":provider::SOURCE_ROUTES,"release_matrix":projection,"preparation":preparation,"installation":installation,
                "provider_invoked":false,"installation_performed":false,"release_qualified":false,
                "next_action":"select exact independently built artifacts/contracts and explicit native request for candidate work; publication, preparation and qualification require their own evidence"});
            Ok(Some(success("doctor", value, outcome)?))
        }
        "inspect" | "plan" => {
            if args.options.contains_key("--native-selection") {
                let selected = provider::Request::from_json_slice(&read_document(
                    &path(args, "--native-selection")?,
                    262_144,
                )?)?;
                let local = local_request(args)?;
                return Ok(Some(local_native(args, &local, &selected, token)?));
            }
            if !args.options.contains_key("--request") {
                return Ok(None);
            }
            let bytes = read_document(&path(args, "--request")?, 1_048_576)?;
            match schema(&bytes)?.as_str() {
                provider::REQUEST_SCHEMA => Ok(Some(native_plan(
                    args,
                    &provider::Request::from_json_slice(&bytes)?,
                    token,
                )?)),
                "flow.local-input-native-request/v1" => {
                    let wrapper: crate::local_input::NativeRequest = serde_json::from_slice(&bytes)
                        .map_err(|_| {
                            error(
                                Code::InvalidInput,
                                "closed local/native request wrapper required",
                            )
                        })?;
                    Ok(Some(local_native(
                        args,
                        &wrapper.request,
                        &wrapper.selection,
                        token,
                    )?))
                }
                _ => Ok(None),
            }
        }
        "approve" | "run" => {
            let bytes = read_document(&path(args, "--plan")?, 16_777_216)?;
            if schema(&bytes)? != provider::PLAN_SCHEMA {
                return Ok(None);
            }
            let plan = provider::Plan::from_json_slice(&bytes)?;
            if args.command == "approve" {
                let by = args
                    .options
                    .get("--approved-by")
                    .and_then(|v| v.to_str())
                    .ok_or_else(|| {
                        error(Code::Usage, "bounded UTF-8 operator declaration required")
                    })?
                    .to_owned();
                let approval = provider::approve(
                    &plan,
                    by,
                    args.acknowledge_trusted_unconfined,
                    args.acknowledge_unqualified_candidate,
                )?;
                provider::save_document(&plan, &path(args, "--output")?, &approval)?;
                Ok(Some(success("approve", approval, Outcome::Complete)?))
            } else {
                let approval: provider::Approval =
                    serde_json::from_slice(&read_document(&path(args, "--approval")?, 16_384)?)
                        .map_err(|_| {
                            error(
                                Code::InvalidInput,
                                "closed supplied native approval required",
                            )
                        })?;
                Ok(Some(success(
                    "run",
                    provider::run(&plan, &approval, token)?,
                    Outcome::Complete,
                )?))
            }
        }
        "status" | "resume" => {
            let root = path(args, "--run-directory")?;
            let bytes =
                read_document(&root.join("operation-plan.json"), 16_777_216).map_err(|mut e| {
                    e.recovery_directory = Some(NativePath::from_path(&root));
                    e
                })?;
            if schema(&bytes)? != provider::PLAN_SCHEMA {
                return Ok(None);
            }
            if args.command == "status" {
                let value = provider::status(&root, token)?;
                let outcome = match value.state {
                    provider::State::Complete => Outcome::Complete,
                    provider::State::RecoveryRequired => Outcome::RecoveryRequired,
                };
                Ok(Some(success("status", value, outcome)?))
            } else {
                if !args.options.contains_key("--native-run-directory") {
                    return Err(error(
                        Code::RecoveryRequired,
                        "native resume requires explicit --native-run-directory from retained provider evidence; no run is guessed or replayed",
                    ));
                }
                Ok(Some(success(
                    "resume",
                    provider::resume(&root, &path(args, "--native-run-directory")?, token)?,
                    Outcome::Complete,
                )?))
            }
        }
        _ => Ok(None),
    }
}
pub(super) fn human_matrix(value: &serde_json::Value) -> String {
    let mut text = format!(
        "Cohort {}: {} green, {} blocked, {} unverified\n",
        value["cohort_id"].as_str().unwrap_or("unknown"),
        value["green"],
        value["blocked"],
        value["unverified"]
    );
    if let Some(rows) = value["rows"].as_array() {
        for row in rows {
            text.push_str(row["declaration"]["id"].as_str().unwrap_or("unknown row"));
            if let Some(cells) = row["cells"].as_array() {
                for cell in cells {
                    text.push_str(&format!(
                        " | {}: {}",
                        cell["dimension"].as_str().unwrap_or("unknown"),
                        cell["observation"]["state"]
                            .as_str()
                            .unwrap_or("unobserved")
                    ));
                }
            }
            text.push('\n');
        }
    }
    text.push_str("Publication, preparation/runtime and positive native qualification require independent evidence. Use --format json for row owners, requirements and next actions.\n");
    text
}
