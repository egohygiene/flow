# Release cohorts and availability

`flow.release-cohort/v1` inventories exact independently versioned artifacts and
meaningful profile/direction/target rows. `flow.release-matrix/v1` reports five
independent observations: contracts, publication, installation, runtime and native
qualification. It does not redefine `flow.compatibility/v1`.

Use `release_cohort::Cohort::from_json_slice`, then `release_cohort::project` with a
trusted `EvidencePort`. The built-in `LocalFiles` observes explicitly selected
canonical executable and contract files, with bounded hashes and cancellation;
it does not invoke a CLI. Installation matching is different from successful
dependency preparation. Model/tool readiness belongs to runtime evidence.

A host evidence port must verify each fact against `subject(row)`, including the
target, direction, options/effects, requirements, release and contract digests.
Returning a boolean copied from JSON or a successful process exit is insufficient.
Publication evidence must identify the exact immutable public artifact and its
trust verification. Qualification must include positive native/domain acceptance,
clean installation and the declared platform/dependency requirements. The API
rejects observations for another subject and passed facts without evidence pins.
The caller chooses this host authority explicitly; an arbitrary port is not an
authenticated verifier merely because it implements a Rust trait.

The source inventory in `examples/release-cohort-source.json` deliberately has no
invented binary digests or qualified rows. Its separate CLI/library/Flow coverage
keeps retained Intel Mac and Windows objectives visible. Expand it from current
provider registries and actual published manifests as release work progresses.
Do not remove advertised rows to improve a percentage. `retained_rows` must match
the row IDs exactly; retain the previous cohort when changing the inventory.

Artifact `release_url` is a location declaration, not proof of publication or
publisher identity. Local checksums establish matching bytes only. This checkpoint
does not build, install, run or release any tool. Deferred execution and final
suite acceptance remain with #88, last; its ledger is unchanged.
