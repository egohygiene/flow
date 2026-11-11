# ADR-0017: Observe native release cohorts without promoting declarations

Status: **Proposed**. Date: 2026-11-11. Relates to #51, #52, #70, #71, #72.

Flow coordinates independently released providers. Source registries, candidate
packages and installed binaries do not establish the same facts. The existing
extension compatibility decision must retain its meaning.

Introduce `flow.release-cohort/v1` and `flow.release-matrix/v1`. Inventory rows
retain missing target/profile/direction coverage. Their exact subject includes
artifact and contract pins, requirements and effects. A trusted host evidence port
returns independently verified publication, installation, contracts, runtime and
qualification observations bound to that subject. Evidence is not deserialized
from cohort JSON. A passed observation requires an evidence digest. Every fact
must pass for an implemented, pinned row to be green. Local file observation can
establish installation and contract bytes only, without launching anything.

Native adapters will translate provider-owned CLI protocols through existing
bounded process supervision. Providers retain their domain algorithms and native
acceptance decisions. Flow retains exact selection, intent, authority, state and
reobservation; no sibling crate dependency or second process engine is introduced.
Candidate contracts may support source development while published-artifact and
qualification cells remain unobserved. They do not advertise a qualified release.

Consequences: source progress remains usable and inspectable without false release
claims. Trusted host port implementations become part of the evidence boundary;
an arbitrary implementation that copies an untrusted assertion is unsuitable.
Checksums bind bytes, not publisher identity. Platform, dependency and native
proof gaps remain visible. No ADR is ratified by this checkpoint.
