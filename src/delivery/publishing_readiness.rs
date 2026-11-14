//! Read-only handoff over current package bytes and historical check snapshots.
//! This report grants no review, native/release, current-QC or upload authority.
use super::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub package_manifest: FilePin,
    pub audio_summary: Option<FilePin>,
    pub timeout_ms: u64,
}
impl Request {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 131_072 {
            return Err(error(
                Code::InvalidInput,
                "publishing readiness request exceeds 128 KiB",
            ));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                Code::InvalidInput,
                "closed publishing readiness request required",
            )
        })?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.publishing-readiness-request/v1"
            || self.timeout_ms == 0
            || self.timeout_ms > 86_400_000
        {
            return Err(error(
                Code::InvalidInput,
                "bounded publishing readiness request required",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    SupplyMetadata,
    SelectDestinationPayload,
    Review,
    AddressReviewChanges,
    SelectAudioPolicy,
    RefreshOwnerCheck,
    ReviewAudioGoal,
    ResolveMissingQc,
    VerifyQualification,
    CheckPlatformRequirements,
    AuthorizePublication,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub kind: ActionKind,
    pub payload_id: Option<String>,
    pub destination: Option<Destination>,
    pub review_scope: Option<ReviewScope>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedAudio {
    pub payload_id: String,
    pub destination: Destination,
    pub recorded_state: String,
    pub policy: Option<Value>,
    pub integrated_loudness: Option<Value>,
    pub true_peak: Option<Value>,
    pub matches_selected_policy_in_record: Option<bool>,
    pub meter_qualified_in_record: bool,
    pub provider_release_qualified_in_summary: bool,
    pub native_record: Option<FilePin>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: String,
    pub package_subject_sha256: String,
    pub package_manifest: FilePin,
    pub payloads: Vec<Payload>,
    pub requested_destinations: Vec<Destination>,
    pub reviews: ReviewSummary,
    pub missing_metadata: Vec<String>,
    pub missing_destination_payloads: Vec<Destination>,
    pub audio_summary: Option<FilePin>,
    pub recorded_audio: Vec<RecordedAudio>,
    pub actions: Vec<Action>,
    pub current_copied_bytes: bool,
    pub current_owner_checks_performed: bool,
    pub release_admission_reverified: bool,
    pub current_platform_compliance_verified: bool,
    pub upload_authority: bool,
    pub evidence_scope: String,
}
fn action(
    kind: ActionKind,
    id: Option<&str>,
    destination: Option<Destination>,
    review_scope: Option<ReviewScope>,
    reason: &str,
) -> Action {
    Action {
        kind,
        payload_id: id.map(str::to_owned),
        destination,
        review_scope,
        reason: reason.into(),
    }
}
fn check_root(pin: &FilePin) -> Result<PathBuf, Error> {
    let file = native::path(&pin.path)?;
    let attempt = file
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "owner snapshot parent required"))?;
    let name = attempt
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("attempt-"))
        .ok_or_else(|| {
            error(
                Code::InvalidInput,
                "explicit native attempt snapshot required",
            )
        })?;
    if file.file_name().and_then(|s| s.to_str()) != Some("domain-acceptance.json")
        || name.len() < 4
        || name.len() > 10
        || !name.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(error(
            Code::InvalidInput,
            "exact retained native domain-acceptance snapshot required",
        ));
    }
    Ok(attempt
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "owner check root required"))?
        .to_path_buf())
}
fn audio_rows(
    summary: &audio_readiness::Summary,
    package: &Status,
    token: &dyn CancellationSignal,
) -> Result<(Vec<RecordedAudio>, Vec<PathBuf>), Error> {
    let preview = summary.evidence_scope == "preview_only; current_owner_checks_not_performed";
    let checked = summary.evidence_scope
        == "owner_rechecked_during_this_attempt; not_a_permanent_or_platform_certificate";
    if summary.schema_version != "flow.audio-readiness-summary/v1"
        || (!preview && !checked)
        || summary.upload_authority
        || summary.current_platform_compliance_verified
        || summary.package_subject_sha256 != package.preview.package_subject_sha256
        || summary.package_manifest_sha256 != package.manifest.sha256
        || summary.reviews != package.preview.reviews
        || summary.missing_metadata != package.preview.missing_metadata
        || summary.missing_destination_payloads != package.preview.missing_destination_payloads
        || summary.rows.len() > 128
    {
        return Err(error(
            Code::StalePlan,
            "audio summary is not bound to the exact current package and declared snapshot scope",
        ));
    }
    let expected = package
        .preview
        .payloads
        .iter()
        .filter(|p| audio_readiness::audio_role(p.role))
        .flat_map(|p| {
            p.destinations
                .iter()
                .map(move |d| audio_readiness::key(&p.id, *d))
        })
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut count = 0;
    let mut rows = vec![];
    let mut protected = vec![];
    for row in &summary.rows {
        if !seen.insert(audio_readiness::key(&row.payload_id, row.destination)) {
            return Err(error(
                Code::InvalidInput,
                "duplicate audio summary payload/destination",
            ));
        }
        let payload = package
            .preview
            .payloads
            .iter()
            .find(|p| {
                p.id == row.payload_id
                    && p.destinations.contains(&row.destination)
                    && audio_readiness::audio_role(p.role)
            })
            .ok_or_else(|| {
                error(
                    Code::InvalidInput,
                    "audio summary names an unassigned payload",
                )
            })?;
        if row.relative != payload.relative
            || row.sha256 != payload.sha256
            || row.byte_count != payload.byte_count
            || row.note.len() > 1024
        {
            return Err(error(
                Code::StalePlan,
                "audio summary belongs to different exact delivery bytes",
            ));
        }
        match row.state.as_str() {
            "archive_master_unchanged" => {
                if !audio_readiness::archive_skip(payload.role, row.destination, &row.note)
                    || row.policy.is_some()
                    || row.integrated_loudness.is_some()
                    || row.true_peak.is_some()
                    || row.matches_selected_policy.is_some()
                    || row.meter_qualified
                    || row.provider_release_qualified
                    || row.native_record.is_some()
                {
                    return Err(error(
                        Code::InvalidInput,
                        "archival preservation cannot infer audio checks or qualification",
                    ));
                }
            }
            "fresh_owner_check_required" => {
                if !preview
                    || row
                        .policy
                        .as_ref()
                        .is_none_or(|p| p["schema"] != "aniflow.audio-delivery-policy/v1")
                    || row.integrated_loudness.is_some()
                    || row.true_peak.is_some()
                    || row.matches_selected_policy.is_some()
                    || row.meter_qualified
                    || row.provider_release_qualified
                    || row.native_record.is_some()
                {
                    return Err(error(
                        Code::InvalidInput,
                        "preview cannot contain observed or qualified audio",
                    ));
                }
            }
            "matches_selected_policy" | "outside_target" | "missing_evidence" => {
                if !checked {
                    return Err(error(
                        Code::InvalidInput,
                        "recorded owner check requires its declared historical scope",
                    ));
                }
                let pin = row.native_record.as_ref().ok_or_else(|| {
                    error(
                        Code::InvalidInput,
                        "recorded audio requires its exact native snapshot pin",
                    )
                })?;
                let record: Value = object(pin, token)?;
                let assessment = native::retained_audio_policy_record(
                    &record,
                    &payload.sha256,
                    payload.byte_count,
                )?;
                let e = &assessment["evaluation"];
                let expected_state = if e["matches_selected_policy"] == true {
                    "matches_selected_policy"
                } else if e["integrated_loudness"]["state"] == "missing_evidence"
                    || e["true_peak"]["state"] == "missing_evidence"
                {
                    "missing_evidence"
                } else {
                    "outside_target"
                };
                if row.state != expected_state
                    || row.policy.as_ref() != Some(&e["policy"])
                    || row.integrated_loudness.as_ref() != Some(&e["integrated_loudness"])
                    || row.true_peak.as_ref() != Some(&e["true_peak"])
                    || row.matches_selected_policy != e["matches_selected_policy"].as_bool()
                    || Some(row.meter_qualified) != e["runtime_qualified"].as_bool()
                {
                    return Err(error(
                        Code::StalePlan,
                        "audio summary differs from its exact retained owner evaluation",
                    ));
                }
                protected.push(check_root(pin)?);
                protected.extend(native::retained_audio_policy_paths(&assessment)?);
                count += 1;
            }
            _ => {
                return Err(error(
                    Code::InvalidInput,
                    "recognized historical audio row state required",
                ));
            }
        }
        rows.push(RecordedAudio {
            payload_id: row.payload_id.clone(),
            destination: row.destination,
            recorded_state: row.state.clone(),
            policy: row.policy.clone(),
            integrated_loudness: row.integrated_loudness.clone(),
            true_peak: row.true_peak.clone(),
            matches_selected_policy_in_record: row.matches_selected_policy,
            meter_qualified_in_record: row.meter_qualified,
            provider_release_qualified_in_summary: row.provider_release_qualified,
            native_record: row.native_record.clone(),
        });
    }
    let matches = summary
        .rows
        .iter()
        .all(|r| r.state == "archive_master_unchanged" || r.matches_selected_policy == Some(true));
    let qualification = summary.rows.iter().any(|r| {
        r.state != "archive_master_unchanged"
            && (!r.meter_qualified || !r.provider_release_qualified)
    });
    if seen != expected
        || count != summary.owner_checks_performed
        || summary.all_selected_audio_goals_match != matches
        || summary.qualification_pending != qualification
    {
        return Err(error(
            Code::InvalidInput,
            "audio summary coverage or aggregate claims differ from the selected rows",
        ));
    }
    Ok((rows, protected))
}
fn observe(
    request: &Request,
    token: &dyn CancellationSignal,
) -> Result<(Report, Vec<PathBuf>), Error> {
    request.validate()?;
    let start = Instant::now();
    let guard =
        || token.is_cancelled() || start.elapsed() >= Duration::from_millis(request.timeout_ms);
    let manifest = native::path(&request.package_manifest.path)?;
    if manifest.file_name().and_then(|s| s.to_str()) != Some("manifest.json") {
        return Err(error(
            Code::InvalidInput,
            "exact delivery manifest.json required",
        ));
    }
    request.package_manifest.current(MAX_DOC, &guard)?;
    let package = super::status(
        manifest
            .parent()
            .ok_or_else(|| error(Code::InvalidInput, "package parent required"))?,
        &request.package_manifest.sha256,
        &guard,
    )?;
    let mut protected = vec![native::path(&package.current_root)?];
    for source in &package.preview.selected_sources {
        protected.push(native::path(&source.file.path)?);
    }
    let recorded_audio = if let Some(pin) = &request.audio_summary {
        protected.push(native::path(&pin.path)?);
        let summary: audio_readiness::Summary = object(pin, &guard)?;
        let (rows, paths) = audio_rows(&summary, &package, &guard)?;
        protected.extend(paths);
        rows
    } else {
        vec![]
    };
    let mut actions = vec![];
    for field in &package.preview.missing_metadata {
        actions.push(action(
            ActionKind::SupplyMetadata,
            None,
            None,
            None,
            &format!("Supply the selected package metadata field: {field}"),
        ));
    }
    for destination in &package.preview.missing_destination_payloads {
        actions.push(action(ActionKind::SelectDestinationPayload,None,Some(*destination),None,"Select an accepted candidate for this requested destination; unsupported requirements need an explicit owner disposition."));
    }
    for scope in &package.preview.reviews.pending {
        actions.push(action(ActionKind::Review,None,None,Some(*scope),"A supplied human review declaration for this exact package is still required; structural checks cannot create it."));
    }
    for scope in &package.preview.reviews.changes_requested {
        actions.push(action(ActionKind::AddressReviewChanges,None,None,Some(*scope),"A supplied review requests changes or rejects this package. Address it and review the new exact content; do not reuse stale approval."));
    }
    for payload in package
        .preview
        .payloads
        .iter()
        .filter(|p| audio_readiness::audio_role(p.role))
    {
        for destination in &payload.destinations {
            let row = recorded_audio
                .iter()
                .find(|r| r.payload_id == payload.id && r.destination == *destination);
            match row.map(|r|r.recorded_state.as_str()){
                Some("archive_master_unchanged")=>{},
                Some(state)=>{
                    if state=="outside_target"{actions.push(action(ActionKind::ReviewAudioGoal,Some(&payload.id),Some(*destination),None,"Review the explicit goal or select a separate source-preserving gain candidate, then obtain fresh exact-final-file QC. No normalization or limiter is implied."));}
                    if state=="missing_evidence"{actions.push(action(ActionKind::ResolveMissingQc,Some(&payload.id),Some(*destination),None,"Resolve unavailable final-file QC quantities through Aniflow; missing measurements cannot become a match."));}
                    actions.push(action(ActionKind::RefreshOwnerCheck,Some(&payload.id),Some(*destination),None,"This report reads retained snapshots only. Perform a fresh explicitly approved owner check for current audio delivery readiness."));
                },
                None=>actions.push(action(ActionKind::SelectAudioPolicy,Some(&payload.id),Some(*destination),None,"Select an explicit audio policy/current-QC request, or an allowed unchanged archival-master choice, using audio-check setup.")),
            }
        }
    }
    actions.push(action(ActionKind::VerifyQualification,None,None,None,"Native/reference-meter/provider release admission is not reverified by this local report. Obtain applicable current evidence through the existing owner and release paths."));
    for destination in package
        .preview
        .requested_destinations
        .iter()
        .filter(|d| **d != Destination::Archive)
    {
        actions.push(action(ActionKind::CheckPlatformRequirements,None,Some(*destination),None,"Recheck the actual current route requirements with exact-file owner preflight; local packaging and recorded targets are not platform compliance."));
        actions.push(action(ActionKind::AuthorizePublication,None,Some(*destination),None,"Actual account posting remains a separate explicitly authorized operation after final review; no upload, URL or publication receipt is inferred."));
    }
    if actions.len() > 1024 {
        return Err(error(
            Code::InvalidInput,
            "publishing readiness action bound",
        ));
    }
    request.package_manifest.current(MAX_DOC, &guard)?;
    if let Some(pin) = &request.audio_summary {
        pin.current(MAX_DOC, &guard)?;
    }
    let report=Report {schema_version:"flow.publishing-readiness/v1".into(),package_subject_sha256:package.preview.package_subject_sha256,package_manifest:request.package_manifest.clone(),payloads:package.preview.payloads,requested_destinations:package.preview.requested_destinations,reviews:package.preview.reviews,missing_metadata:package.preview.missing_metadata,missing_destination_payloads:package.preview.missing_destination_payloads,audio_summary:request.audio_summary.clone(),recorded_audio,actions,current_copied_bytes:true,current_owner_checks_performed:false,release_admission_reverified:false,current_platform_compliance_verified:false,upload_authority:false,evidence_scope:"current local copied bytes and correlated historical declarations/snapshots only; not current QC, authenticated human review, runtime/release qualification or publication authority".into()};
    Ok((report, protected))
}
/// Read-only current package identity and historical evidence correlation.
pub fn report(request: &Request, token: &dyn CancellationSignal) -> Result<Report, Error> {
    Ok(observe(request, token)?.0)
}
/// Writes a new report outside the package and selected owner/evidence scope.
pub fn save(
    request: &Request,
    output: &Path,
    token: &dyn CancellationSignal,
) -> Result<Report, Error> {
    let (report, protected) = observe(request, token)?;
    let output = command::absolute(output)?;
    if protected
        .iter()
        .any(|p| output.starts_with(p) || p.starts_with(&output))
    {
        return Err(error(
            Code::InvalidInput,
            "publishing report output overlaps package, originals or selected owner evidence",
        ));
    }
    command::write_document(&output, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_preserves_review_gaps_and_cannot_claim_current_qc_or_write_into_the_package() {
        let (_temp, setup) = crate::delivery::audio_setup::tests::archive_fixture();
        let request = Request {
            schema_version: "flow.publishing-readiness-request/v1".into(),
            package_manifest: setup.package_manifest,
            audio_summary: None,
            timeout_ms: 10_000,
        };
        let value = report(&request, &|| false).unwrap();
        assert!(value.current_copied_bytes);
        assert!(
            !value.current_owner_checks_performed
                && !value.release_admission_reverified
                && !value.current_platform_compliance_verified
                && !value.upload_authority
        );
        assert!(
            value
                .actions
                .iter()
                .any(|a| matches!(a.kind, ActionKind::Review)
                    && a.review_scope == Some(ReviewScope::Words))
        );
        assert!(
            value
                .actions
                .iter()
                .any(|a| matches!(a.kind, ActionKind::SelectAudioPolicy))
        );
        let output = native::path(&request.package_manifest.path)
            .unwrap()
            .parent()
            .unwrap()
            .join("report.json");
        assert!(save(&request, &output, &|| false).is_err());
        assert!(!output.exists());
    }
    #[test]
    fn preview_summary_cannot_invent_meters_and_payload_source_ambiguity_refuses() {
        let (_temp, setup) = crate::delivery::audio_setup::tests::archive_fixture();
        let manifest = native::path(&setup.package_manifest.path).unwrap();
        let package = super::super::status(
            manifest.parent().unwrap(),
            &setup.package_manifest.sha256,
            &|| false,
        )
        .unwrap();
        let p = &package.preview.payloads[0];
        let mut summary = audio_readiness::Summary {
            schema_version: "flow.audio-readiness-summary/v1".into(),
            package_subject_sha256: package.preview.package_subject_sha256.clone(),
            package_manifest_sha256: package.manifest.sha256.clone(),
            rows: vec![audio_readiness::Row {
                payload_id: p.id.clone(),
                destination: Destination::Archive,
                relative: p.relative.clone(),
                sha256: p.sha256.clone(),
                byte_count: p.byte_count,
                state: "fresh_owner_check_required".into(),
                policy: Some(json!({"schema":"aniflow.audio-delivery-policy/v1"})),
                integrated_loudness: None,
                true_peak: None,
                matches_selected_policy: None,
                meter_qualified: false,
                provider_release_qualified: false,
                native_record: None,
                note: "Synthetic unmeasured preview".into(),
            }],
            reviews: package.preview.reviews.clone(),
            missing_metadata: package.preview.missing_metadata.clone(),
            missing_destination_payloads: vec![],
            all_selected_audio_goals_match: false,
            owner_checks_performed: 0,
            qualification_pending: true,
            evidence_scope: "preview_only; current_owner_checks_not_performed".into(),
            upload_authority: false,
            current_platform_compliance_verified: false,
        };
        audio_rows(&summary, &package, &|| false).unwrap();
        summary.rows[0].meter_qualified = true;
        assert!(audio_rows(&summary, &package, &|| false).is_err());
        let mut ambiguous = package.preview.clone();
        ambiguous.selected_sources[0].file.sha256 = "f".repeat(64);
        assert!(super::super::validate_payload_ledger(&ambiguous).is_err());
    }
}
