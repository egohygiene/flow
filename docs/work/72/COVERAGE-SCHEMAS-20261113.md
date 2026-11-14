# Requested release coverage schema reconciliation

November 13, 2026 (America/New_York). During the maintainer-authorized whole-queue
merge, Flow #131 is narrowed to its unique requested-coverage schema work after
#129. Its two closed release-coverage schemas and compiled descriptor bindings
remain, preserving the original requested tool/interface/target/profile contract.
The schemas parse as JSON and the Rust source is normalized with Rustfmt 1.85.
No product compilation, schema execution, Rust tests or hosted CI is claimed.

The old diagnostic implementation, five diagnostic/health schema documents,
command adapter, Task aliases and runbook are superseded by merged #136's newer
metadata-only support owner. They are not restored or advertised as the newer
support wire formats. Existing support commands, aliases and support schemas are
retained without change. The separate scenario implementation remains #135.
#21/#53/#72 acceptance and #88's final pass remain open; no release or runtime
qualification follows from this reconciliation.
