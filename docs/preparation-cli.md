# Explicit local preparation

The main binary uses the same SDK as the provisional `flow-ai-setup` source
utility. Supply an existing private, empty directory (Unix mode 0700) and an
explicit `flow.ai-local-setup/v1` request using the existing
`ai_preparation::setup::Request` fields. Existing setup profiles and license,
memory, observation, context, cost and deadline limits remain authoritative.

```text
flow prepare-assets --request /private/selected-assets.json --output-directory /private/setup --format json
flow preparation-status --preparation-directory /private/setup --format json
flow doctor --preparation-directory /private/setup --installation-root /private/suite-version --receipt-sha256 EXACT_RECEIPT_SHA --format json
```

These commands observe explicitly selected existing files, write immutable private
documents, and report actionable missing/changed/incompatible states. They never
download weights, start services, probe GPUs, invoke inference or fall back to a
cloud account. Supply runtime and model licenses separately. Logical file sizes
do not establish physical storage, memory enforcement or offline service behavior.

The native request's optional `preparation` is a `FilePin` of the existing SDK's
`setup.json` marker: exact absolute native path, SHA-256 and byte count. Omitting
it preserves existing request and plan encodings. A selected marker is included
in plan identity; planning and execution reobserve the closed document set and
the selected assets. Only disabled or currently matched-byte preparation states
can proceed, and output must be disjoint from those sources and documents.
Preparation never authorizes execution. The provider plan, trusted-unconfined
approval, release admission and actual backend evidence remain separate.

Doctor reports an optional exact suite installation receipt independently from
optional model preparation and the release matrix. It does not discover ambient
PATH entries or turn a successful installation copy into native readiness.

Source checkpoint only: CLI, setup, installer, native work, builds, validators and
tests have not been executed for this sprint. Hardware/runtime qualification and
the final sweep remain tracked by the existing issues; Flow #88 remains last.
