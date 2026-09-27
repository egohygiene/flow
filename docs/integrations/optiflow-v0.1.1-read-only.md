# Optiflow v0.1.1 read-only release adapter

Flow #50 pins the independently verified, signed Optiflow v0.1.1 release
(`b82599a2231e997d42fd9f26f4b59587f4ae14cf`). Use the
[`OptiflowReadOnlyAdapter`](../../src/optiflow.rs) public library API with
the exact executable extracted from the release bundle. The adapter never
downloads or searches `PATH` at runtime.

## Release lock

The release's `binary-v0.1.1.tar.gz` bundle has SHA-256
`f576131f2695a218fabfaa5167fd16f17641c267194254a9e624303a8cf23e07`.
The target archives and extracted executables are pinned separately:

| Target | Archive SHA-256 | Executable SHA-256 | Local execution |
| --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `5e8a0eea84f5ab55fe75fc8a8abc2e9d36e3512537b754262b1f34a9c40aae31` | `46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280` | synthetic fixture passed on Linux/Rust 1.85 |
| `aarch64-apple-darwin` | `f83fa18a98e99198e535e37dd93e1584024985df4960f2defd8124d52e656ab1` | `2a2411dbbafd5c22a2b398f13eaf4f0be84e5dedb8f4c9d80f7df418522c6ca3` | archive inspected; native run pending |
| `x86_64-apple-darwin` | `71f1cc9be7ec7d3373807e012114bb64aba62f550fd9da6b4ddccda62397c694` | `a3097b5060ae05a361c97dd004857e7152a019f3f11dc0459b993183a6cf2135` | archive inspected; native run pending |

Verify the bundle and target archive SHA-256 before extraction. `probe`
checks a regular absolute executable's exact target digest and a bounded
`optiflow 0.1.1` version response. The release evidence identifies the
source revision above; digest matching is byte identity, not a new signature
verification performed by this adapter.

## Interface and effects

| Capability | v0.1.1 state | Source effect | Local evidence effect |
| --- | --- | --- | --- |
| Scan | available read-only | recursive read and hashing within one root and filesystem | Optiflow state, run, report, policy, artifact-set marker |
| Report | available read-only | reads committed provider state | re-verifies report set; Flow receipt |
| Plan exact duplicates | available read-only | review suggestions only | immutable review plan and marker; Flow receipt |
| Dry-run | unsupported by pinned release | none | typed refusal |
| Quarantine | unsupported by pinned release | none | typed refusal |
| Restore | unsupported by pinned release | none | typed refusal |
| Finalize | unsupported by pinned release | none | typed refusal |

The direct child uses a cleared environment, `--no-config`, `--no-probe`,
`--no-follow-symlinks`, and `--stay-on-filesystem`. No network, media
probe, source mutation, deletion, approval, or physical savings is claimed.
Its trusted executable is not an OS sandbox; host code can still reach
resources the OS permits. The selected evidence directory must be disjoint
from the source. The child has a 120-second scan deadline and 30-second
report/plan deadline, 16 MiB stdout, 64 KiB stderr, and 16 MiB per artifact
read limits. A larger run refuses; callers can select a smaller source.

The native interface is `optiflow.command-result.v1`, exit classes
`success` (0), `partial_success` (3), `stale_state` (5), and other
provider-declared failures. Accepted domain schemas are
`optiflow.run.v5`, `optiflow.report.v6`, `optiflow.plan.v5`,
`optiflow.effective-policy.v1`, and `optiflow.artifact-set.v1`.
Flow checks outcome against actual exit, requires explicit complete or
partial coverage, re-hashes committed marker members, verifies native path
containment and source bindings, and retains typed provider diagnostics.
Human text is never interpreted as machine output.

## Library use

```rust,no_run
use std::path::Path;
use flow::optiflow::{OptiflowReadOnlyAdapter, ReadOnlyReceipt};

# fn main() -> Result<(), flow::optiflow::AdapterError> {
let adapter = OptiflowReadOnlyAdapter::probe(
    Path::new("/absolute/path/to/verified/optiflow")
)?;
let (receipt, receipt_path) = adapter.inspect(
    Path::new("/selected/source"),
    Path::new("/separate/local/optiflow-state"),
)?;
assert!(receipt.plan_sha256.is_some());
let reopened = ReadOnlyReceipt::load(&receipt_path)?;
assert!(!reopened.needs_recovery());
# Ok(())
# }
```

The public API completes scan → report → review plan sequentially. It
records `flow.optiflow-read-only-receipt/v1` at
`<state>/flow-optiflow-v0.1.1/<attempt>.json` before launch and after
each validated transition. The [receipt schema](../../schemas/optiflow-read-only-receipt-v1.schema.json)
includes target digest, exact tagged argv, cleared-environment assumptions,
source root filesystem and file IDs, run and set IDs, provider diagnostics,
member sizes and SHA-256/BLAKE3 digests, plan digest, and coverage. Approval
and physical savings are null; duplicate logical bytes are a separate
observation. Native plan `keep_path` is a suggestion, never an authority
record.

If a process or host stops between transitions, reopen the receipt. Scanning,
reporting, or planning status requires operator inspection of local provider
state; no output file alone marks a Flow attempt complete. A timeout or local
I/O uncertainty is also marked for inspection. Reopening checks recorded
artifact bytes again and refuses changed evidence. An explicit new invocation
can start a fresh attempt after review; the adapter never automatically
promotes or retries an uncertain attempt. The local receipt and state are
trusted local audit data, not cryptographic authentication.

## Reproducing the clean-room fixture

From a fresh Flow checkout with Rust 1.85, verify and extract the official
release archive, then run:

```bash
FLOW_OPTIFLOW_V011_EXECUTABLE="/absolute/path/to/release/optiflow" \
  cargo test --locked --test optiflow_read_only
```

The test creates only disposable text files with duplicates and Unicode
paths, checks no source byte change, and probes refusal and interrupted
evidence. It never scans personal media. Without the variable, the local
release execution cases are skipped while the executable mismatch test
still runs. The generic Flow JSON Lines runner and durable graph contracts
remain separate. Flow #53 will compose released adapters into the suite
CLI; Flow #73 handles the separately qualified mutation release.
