//! Explicit adjusted-media selection from retained native owners. No processing,
//! fresh QC, release admission, human approval or upload occurs during packaging.
use super::*;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub processing_plan: FilePin,
    pub artifact_id: String,
    pub qc_plan: FilePin,
    pub policy_plan: FilePin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    plan: FilePin,
    configuration: FilePin,
    receipt: FilePin,
    acceptance: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lineage {
    schema_version: String,
    payload_id: String,
    role: Role,
    pilot_plan_sha256: String,
    selection: Selection,
    file: FilePin,
    anchor: FilePin,
    processing: History,
    qc: History,
    policy: History,
    policy_sha256: String,
    evidence_scope: String,
    native_release_admission_reverified: bool,
    current_owner_rechecked: bool,
    upload_authority: bool,
}
const SCOPE: &str = "retained_native_processing_QC_and_policy_snapshots; current_QC_release_and_human_review_not_reverified";
struct Loaded {
    history: History,
    plan: native::Plan,
    receipt: native::Receipt,
    configuration: Value,
    domain: Value,
}
pub(super) struct Resolved {
    pub(super) file: FilePin,
    pub(super) source_state: String,
    pub(super) members: Vec<Member>,
    pub(super) protected: Vec<PathBuf>,
}
fn profiles(role: Role) -> Result<(&'static str, &'static str, &'static str), Error> {
    match role {
        Role::AudioDerivative => Ok((
            "fixed-audio-gain-v1",
            "final-lossless-audio-qc-v1",
            "audio_derivative",
        )),
        Role::VideoDelivery => Ok(("gain-video-v1", "final-video-audio-qc-v1", "video_delivery")),
        _ => Err(error(
            Code::InvalidInput,
            "adjusted native media can only be an audio/video derivative; retain the accepted clean master separately",
        )),
    }
}
fn validate_loaded(v: &Loaded) -> Result<(), Error> {
    let mut unsigned = v.plan.clone();
    unsigned.plan_sha256.clear();
    if v.plan.schema_version != native::PLAN_SCHEMA
        || hash(&unsigned)? != v.plan.plan_sha256
        || v.plan.row.tool != crate::release_cohort::Tool::Aniflow
        || v.plan.configuration != v.history.configuration
        || v.receipt.schema_version != native::RECEIPT_SCHEMA
        || v.receipt.plan_sha256 != v.plan.plan_sha256
        || v.receipt.row_subject_sha256 != v.plan.row_subject_sha256
        || v.receipt.state != native::State::Complete
        || v.receipt.release_qualified
        || v.receipt.creative_approval_inferred
        || v.receipt.publication_authority
        || v.receipt.acceptance_file != v.history.acceptance.path
        || v.receipt.acceptance_sha256 != v.history.acceptance.sha256
        || v.receipt.artifacts.len() != 1
    {
        return Err(error(
            Code::RecoveryRequired,
            "exact retained native processing/QC/policy history required",
        ));
    }
    let root = native::path(&v.plan.request.run_directory)?;
    if native::path(&v.history.receipt.path)? != root.join("accepted.json")
        || !native::path(&v.receipt.native_run)?.starts_with(root.join("native"))
        || native::path(&v.receipt.native_run)? == root.join("native")
    {
        return Err(error(
            Code::RecoveryRequired,
            "retained native history path differs",
        ));
    }
    Ok(())
}
fn current(pin: &FilePin, token: &dyn CancellationSignal) -> Result<Loaded, Error> {
    let plan = native::Plan::from_json_slice(&read(pin, MAX_DOC, token)?)?;
    plan.require_current(token)?;
    let root = native::path(&plan.request.run_directory)?;
    let receipt = native::observe_receipt(&root, &plan.plan_sha256, token)?
        .receipt
        .ok_or_else(|| {
            error(
                Code::RecoveryRequired,
                "complete retained native processing/QC/policy receipt required",
            )
        })?;
    let history = History {
        plan: pin.clone(),
        configuration: plan.configuration.clone(),
        receipt: FilePin::observe(
            &NativePath::from_path(&root.join("accepted.json")),
            MAX_DOC,
            token,
        )?,
        acceptance: FilePin::observe(&receipt.acceptance_file, MAX_DOC, token)?,
    };
    let value = Loaded {
        configuration: object(&history.configuration, token)?,
        domain: object(&history.acceptance, token)?,
        history,
        plan,
        receipt,
    };
    validate_loaded(&value)?;
    Ok(value)
}
fn handoff(v: &Loaded) -> Result<Value, Error> {
    native::retained_audio_processing_record(&v.plan, &v.receipt, &v.configuration, &v.domain)
}
fn normalized_file(value: &Value, file: &FilePin) -> Result<bool, Error> {
    Ok(
        value["locator"].as_str() == native::path(&file.path)?.to_str()
            && value["sha256"] == file.sha256
            && value["byte_count"].as_u64() == Some(file.byte_count),
    )
}
fn bind(
    role: Role,
    selection: &Selection,
    p: &Loaded,
    q: &Loaded,
    c: &Loaded,
) -> Result<(FilePin, String), Error> {
    let (processing, qc, purpose) = profiles(role)?;
    if p.plan.row.profile != processing
        || q.plan.row.profile != qc
        || c.plan.row.profile != "audio-policy-check-v1"
        || [q, c].iter().any(|v| {
            v.plan.provider != p.plan.provider
                || v.plan.descriptor != p.plan.descriptor
                || v.plan.request.cohort_sha256 != p.plan.request.cohort_sha256
        })
    {
        return Err(error(
            Code::InvalidInput,
            "matching processing/final-QC/policy profiles and exact provider cohort required",
        ));
    }
    let processed = handoff(p)?;
    let measured = handoff(q)?;
    let artifact = &processed["payload"]["artifacts"][0];
    let file = p.receipt.artifacts[0].clone();
    if artifact["id"] != selection.artifact_id
        || artifact["purpose"] != purpose
        || !normalized_file(&artifact["file"], &file)?
        || q.plan.sources != vec![file.clone()]
        || c.plan.sources != vec![file.clone()]
    {
        return Err(error(
            Code::InvalidInput,
            "adjusted selection/QC/policy must refer to the same exact accepted final media",
        ));
    }
    let assessment =
        native::retained_audio_policy_record(&c.domain, &file.sha256, file.byte_count)?;
    let input = &assessment["request"]["input"];
    let proc_root = native::path(&p.receipt.native_run)?;
    let qc_root = native::path(&q.receipt.native_run)?;
    let qc_file = &q.receipt.artifacts[0];
    if assessment["request"] != c.configuration
        || assessment["evaluation"]["matches_selected_policy"] != true
        || !normalized_file(&assessment["subject"]["file"], &file)?
        || assessment["subject"]["facts"]["qc"] != measured["payload"]["report"]
        || input["qc_directory"].as_str() != qc_root.to_str()
        || input["qc_receipt_sha256"] != qc_file.sha256
        || c.receipt.native_intent_sha256 != assessment["assessment_sha256"].as_str().unwrap_or("")
    {
        return Err(error(
            Code::InvalidInput,
            "selected-policy snapshot must match the exact final bytes and selected final-QC report",
        ));
    }
    let expected_kind = if role == Role::AudioDerivative {
        "accepted_audio_with_qc"
    } else {
        "accepted_video_with_qc"
    };
    if input["kind"] != expected_kind {
        return Err(error(
            Code::InvalidInput,
            "encoded video requires its own encoded-file QC; WAV checks cannot substitute",
        ));
    }
    if role == Role::AudioDerivative {
        if input["subject"]["kind"] != "accepted_gain_candidate"
            || input["subject"]["run_directory"].as_str() != proc_root.to_str()
            || q.configuration["qc"]["subject"] != input["subject"]
        {
            return Err(error(
                Code::InvalidInput,
                "gain WAV source/QC subject differs",
            ));
        }
    } else if input["run_directory"].as_str() != proc_root.to_str()
        || input["artifact_id"] != selection.artifact_id
        || q.configuration["qc"]["source_run_directory"].as_str() != proc_root.to_str()
        || q.configuration["qc"]["artifact_id"] != selection.artifact_id
    {
        return Err(error(
            Code::InvalidInput,
            "encoded video source/QC artifact differs",
        ));
    }
    if c.domain["source_pins"]
        != serde_json::to_value(&c.plan.sources)
            .map_err(|_| error(Code::InvalidInput, "policy source encoding"))?
        || c.receipt.artifacts.len() != 1
    {
        return Err(error(
            Code::RecoveryRequired,
            "policy plan and acceptance source pins differ",
        ));
    }
    let policy_sha = assessment["evaluation"]["policy_sha256"]
        .as_str()
        .filter(|s| release_cohort::is_digest(s))
        .ok_or_else(|| {
            error(
                Code::RecoveryRequired,
                "exact selected policy identity required",
            )
        })?
        .to_owned();
    Ok((file, policy_sha))
}
fn records(h: &History) -> [(&'static str, &FilePin); 4] {
    [
        ("plan.json", &h.plan),
        ("configuration.json", &h.configuration),
        ("receipt.json", &h.receipt),
        ("acceptance.json", &h.acceptance),
    ]
}
fn state(lineage: &Lineage) -> Result<String, Error> {
    Ok(format!("accepted_adjusted_native:{}", hash(lineage)?))
}
fn prefix(id: &str) -> String {
    format!("evidence/adjusted/{id}")
}
pub(super) fn resolve(
    selection: &Selection,
    selected: &super::Selection,
    context: &Context,
    token: &dyn CancellationSignal,
) -> Result<Resolved, Error> {
    profiles(selected.role)?;
    let p = current(&selection.processing_plan, token)?;
    let q = current(&selection.qc_plan, token)?;
    let c = current(&selection.policy_plan, token)?;
    let (file, policy_sha256) = bind(selected.role, selection, &p, &q, &c)?;
    let anchor = p
        .plan
        .sources
        .first()
        .ok_or_else(|| error(Code::InvalidInput, "original processing source missing"))?
        .clone();
    let accepted = context.catalog.artifacts.iter().any(|a| {
        a.file == anchor
            && (selected.role != Role::VideoDelivery
                || a.stage == "video"
                    && ["clean_archival_master", "destination_copy"].contains(&a.purpose.as_str()))
    });
    if !accepted
        && !(selected.role == Role::AudioDerivative
            && context.catalog.original_audio.as_ref() == Some(&anchor))
    {
        return Err(error(
            Code::InvalidInput,
            "adjusted media must originate from this pilot's exact accepted video/audio or selected original audio",
        ));
    }
    let lineage = Lineage {
        schema_version: "flow.adjusted-delivery-lineage/v1".into(),
        payload_id: selected.id.clone(),
        role: selected.role,
        pilot_plan_sha256: context.catalog.plan_sha256.clone(),
        selection: selection.clone(),
        file: file.clone(),
        anchor,
        processing: p.history.clone(),
        qc: q.history.clone(),
        policy: c.history.clone(),
        policy_sha256,
        evidence_scope: SCOPE.into(),
        native_release_admission_reverified: false,
        current_owner_rechecked: false,
        upload_authority: false,
    };
    let at = prefix(&selected.id);
    let mut members = vec![];
    let mut protected = vec![];
    for (label, v) in [("processing", &p), ("qc", &q), ("policy", &c)] {
        for (name, source) in records(&v.history) {
            members.push(Member::Copy {
                relative: format!("{at}/{label}/{name}"),
                source: source.clone(),
            });
        }
        protected.push(native::path(&v.plan.request.run_directory)?);
        protected.extend(if label == "policy" {
            native::audio_policy_protected_paths(&v.plan.request)?
        } else {
            native::audio_processing_protected_paths(&v.plan.request, &v.plan.row.profile)?
        });
        if label != "processing" {
            members.push(Member::Copy {
                relative: format!("{at}/{label}/artifact.json"),
                source: v.receipt.artifacts[0].clone(),
            });
        }
    }
    let source_state = state(&lineage)?;
    members.push(Member::Generated {
        relative: format!("{at}/lineage.json"),
        data: bytes(&lineage)?,
    });
    Ok(Resolved {
        file,
        source_state,
        members,
        protected,
    })
}
fn copied(
    root: &Path,
    relative: &str,
    pin: &FilePin,
    files: &[OwnedFile],
    token: &dyn CancellationSignal,
) -> Result<Vec<u8>, Error> {
    if !files
        .iter()
        .any(|f| f.relative == relative && f.sha256 == pin.sha256 && f.byte_count == pin.byte_count)
    {
        return Err(error(
            Code::RecoveryRequired,
            "adjusted history copy differs from its retained pin",
        ));
    }
    let copy = FilePin {
        path: NativePath::from_path(&root.join(relative)),
        sha256: pin.sha256.clone(),
        byte_count: pin.byte_count,
    };
    read(&copy, MAX_DOC, token)
}
fn portable(
    root: &Path,
    at: &str,
    h: &History,
    files: &[OwnedFile],
    token: &dyn CancellationSignal,
) -> Result<Loaded, Error> {
    fn decode<T: for<'de> Deserialize<'de>>(b: &[u8]) -> Result<T, Error> {
        serde_json::from_slice(b).map_err(|_| {
            error(
                Code::RecoveryRequired,
                "closed retained adjusted-history copy required",
            )
        })
    }
    let value = Loaded {
        history: h.clone(),
        plan: decode(&copied(
            root,
            &format!("{at}/plan.json"),
            &h.plan,
            files,
            token,
        )?)?,
        configuration: decode(&copied(
            root,
            &format!("{at}/configuration.json"),
            &h.configuration,
            files,
            token,
        )?)?,
        receipt: decode(&copied(
            root,
            &format!("{at}/receipt.json"),
            &h.receipt,
            files,
            token,
        )?)?,
        domain: decode(&copied(
            root,
            &format!("{at}/acceptance.json"),
            &h.acceptance,
            files,
            token,
        )?)?,
    };
    validate_loaded(&value)?;
    Ok(value)
}
/// Inspect only copied history. Original/native/tool paths are never observed.
pub(super) fn validate_package(
    root: &Path,
    request: &Request,
    preview: &Preview,
    files: &[OwnedFile],
    token: &dyn CancellationSignal,
) -> Result<(), Error> {
    if request
        .payloads
        .iter()
        .any(|s| matches!(&s.source, Source::AdjustedNative { .. }))
        && !preview.payloads.iter().any(|p| p.role == Role::CleanMaster)
    {
        return Err(error(
            Code::RecoveryRequired,
            "adjusted packages must retain the accepted clean archival master",
        ));
    }
    for selected in &request.payloads {
        let Source::AdjustedNative { selection } = &selected.source else {
            continue;
        };
        if !super::token(&selected.id) {
            return Err(error(
                Code::RecoveryRequired,
                "safe adjusted payload identity required",
            ));
        }
        let at = prefix(&selected.id);
        let name = format!("{at}/lineage.json");
        let member = files
            .iter()
            .find(|f| f.relative == name)
            .ok_or_else(|| error(Code::RecoveryRequired, "adjusted lineage member missing"))?;
        let pin = FilePin {
            path: NativePath::from_path(&root.join(&name)),
            sha256: member.sha256.clone(),
            byte_count: member.byte_count,
        };
        let lineage: Lineage = object(&pin, token)?;
        let payload = preview
            .payloads
            .iter()
            .find(|p| p.id == selected.id)
            .ok_or_else(|| error(Code::RecoveryRequired, "adjusted payload ledger missing"))?;
        if lineage.schema_version != "flow.adjusted-delivery-lineage/v1"
            || lineage.payload_id != selected.id
            || lineage.role != selected.role
            || payload.role != selected.role
            || lineage.pilot_plan_sha256 != preview.plan_sha256
            || lineage.selection != *selection
            || lineage.file.sha256 != payload.sha256
            || lineage.file.byte_count != payload.byte_count
            || payload.source_state != state(&lineage)?
            || payload.relative != format!("deliver/{}/{}", selected.id, selected.file_name)
            || payload.destinations != selected.destinations
            || preview
                .selected_sources
                .iter()
                .find(|s| s.payload_id == selected.id)
                .map(|s| &s.file)
                != Some(&lineage.file)
            || lineage.processing.plan != selection.processing_plan
            || lineage.qc.plan != selection.qc_plan
            || lineage.policy.plan != selection.policy_plan
            || lineage.evidence_scope != SCOPE
            || lineage.native_release_admission_reverified
            || lineage.current_owner_rechecked
            || lineage.upload_authority
        {
            return Err(error(
                Code::RecoveryRequired,
                "adjusted payload/history/review scope differs",
            ));
        }
        let p = portable(
            root,
            &format!("{at}/processing"),
            &lineage.processing,
            files,
            token,
        )?;
        let q = portable(root, &format!("{at}/qc"), &lineage.qc, files, token)?;
        let c = portable(root, &format!("{at}/policy"), &lineage.policy, files, token)?;
        let (file, policy) = bind(selected.role, selection, &p, &q, &c)?;
        if file != lineage.file
            || p.plan.sources.first() != Some(&lineage.anchor)
            || policy != lineage.policy_sha256
        {
            return Err(error(
                Code::RecoveryRequired,
                "adjusted source/policy lineage differs",
            ));
        }
        for (label, v) in [("qc", &q), ("policy", &c)] {
            let raw = copied(
                root,
                &format!("{at}/{label}/artifact.json"),
                &v.receipt.artifacts[0],
                files,
                token,
            )?;
            let actual: Value = serde_json::from_slice(&raw).map_err(|_| {
                error(
                    Code::RecoveryRequired,
                    "retained QC/policy artifact malformed",
                )
            })?;
            let expected = if label == "qc" {
                handoff(v)?["payload"]["report"].clone()
            } else {
                native::retained_audio_policy_record(&v.domain, &file.sha256, file.byte_count)?
            };
            if actual != expected {
                return Err(error(
                    Code::RecoveryRequired,
                    "copied QC/policy artifact differs from its captured owner snapshot",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjusted_roles_cannot_replace_clean_sources_or_omit_final_qc() {
        assert!(profiles(Role::CleanMaster).is_err());
        assert!(profiles(Role::OriginalAudio).is_err());
        let pin = FilePin {
            path: NativePath::from_path(Path::new("/synthetic/plan.json")),
            sha256: "a".repeat(64),
            byte_count: 100,
        };
        let selection = Selection {
            processing_plan: pin.clone(),
            artifact_id: "gain_audio".into(),
            qc_plan: pin.clone(),
            policy_plan: pin,
        };
        let mut source = json!({"kind":"adjusted_native","selection":selection});
        source["selection"]
            .as_object_mut()
            .unwrap()
            .remove("qc_plan");
        assert!(serde_json::from_value::<Source>(source).is_err());
    }
    #[test]
    fn a_new_qc_history_invalidates_review_even_when_media_bytes_match() {
        let pin = FilePin {
            path: NativePath::from_path(Path::new("/synthetic/retained.json")),
            sha256: "a".repeat(64),
            byte_count: 100,
        };
        let history = History {
            plan: pin.clone(),
            configuration: pin.clone(),
            receipt: pin.clone(),
            acceptance: pin.clone(),
        };
        let selection = Selection {
            processing_plan: pin.clone(),
            artifact_id: "gain_audio".into(),
            qc_plan: pin.clone(),
            policy_plan: pin.clone(),
        };
        let mut lineage = Lineage {
            schema_version: "flow.adjusted-delivery-lineage/v1".into(),
            payload_id: "song".into(),
            role: Role::AudioDerivative,
            pilot_plan_sha256: "b".repeat(64),
            selection,
            file: pin.clone(),
            anchor: pin,
            processing: history.clone(),
            qc: history.clone(),
            policy: history,
            policy_sha256: "c".repeat(64),
            evidence_scope: SCOPE.into(),
            native_release_admission_reverified: false,
            current_owner_rechecked: false,
            upload_authority: false,
        };
        let before = state(&lineage).unwrap();
        let media = lineage.file.clone();
        lineage.qc.acceptance.sha256 = "d".repeat(64);
        assert_eq!(lineage.file, media);
        assert_ne!(state(&lineage).unwrap(), before);
        let old = hash(&before).unwrap();
        let new = hash(&state(&lineage).unwrap()).unwrap();
        let review = Review {
            schema_version: "flow.delivery-review/v1".into(),
            package_subject_sha256: old,
            scope: ReviewScope::Audio,
            decision: Decision::Accept,
            reviewed_by: "synthetic fixture".into(),
            reviewed_at: "fixture only".into(),
            notes: "not an actual approval".into(),
        };
        assert!(reviews(&[review], &new).is_err());
    }
}
