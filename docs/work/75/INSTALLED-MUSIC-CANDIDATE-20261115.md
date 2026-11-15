# Installed native Aniflow checkpoint — 2026-11-15

Relates to Flow #51, #75 and #71. The November 14 sprint continued past midnight
UTC. The final fresh observation passes **18 installed Flow CLI calls**, plus
three existing installation/acknowledgement/status calls. No Flow product build
or source change was required.

The exact [direct report](evidence/20261115/direct.json) has SHA-256
`dde28963f3b5f4a516db3cc6c51c9210c3e5063af7700566a32b873b1c225859`.
The [installation report](evidence/20261115/installation.json) is
`e18ea333b96000cd48d5934c5a57eaf1be77f4a1077ae86a64f33c6459fdbd49`.
[Observer source bindings](evidence/20261115/observer-source.json) retain each
measured helper hash and the 37 passing focused admission/completion/inventory
cases, including the unchanged historical v1's 15 cases.

| Subject | Measured identity and scope |
| --- | --- |
| Flow | Original package `62ba25f3dd056b48fdfaf75a83b7f0dbdd3f29524b5ddc471b514246b85ccdbb`, executable `0de124114f96cab0256f09735b68ca56eecab646d608c051f1651892c20585e4`; actual build source `69eafaf74af8891bde1c7a769fb57dbca815476c`, two isolated Rust 1.85 release builds retained from the prior checkpoint |
| Aniflow | Published draft source `e73d35cfa167b0966f24df3d9662d26e2146de17`, executable `e1c6f8ac229cf54d1b921dd521f692b1c420fccbb2a9b6d3213f173eb1539a17`; actual Rust 1.85 optimized-dev capture, reused cache, zero independent rebuild claim |
| Renderflow | Published draft source `848984ab363058b2b765f4056ed01bde9f2b54d6`, executable `61ed02b8a0b5396ea1685c3131ae29f75f943a355668a98106e93fda0e28b942`; actual Rust 1.94 Debian-host release capture; no Ubuntu or native-matrix qualification |
| Optiflow | Actual immutable v0.1.1 executable `46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280`, source `b82599a2231e997d42fd9f26f4b59587f4ae14cf`; original release bundle independently pinned |

The [product-input comparison](evidence/20261115/flow-product-input-equivalence.json)
binds selected source `2b18f11e8820cfb117c26651e28837e624cb6ef6` to the actual older
Flow build: all 30 changed paths are documentation/tools, with no changed runtime
input or embedded documentation/tool file. This does not relabel that executable
as a fresh build at the later observer tip or make the new provider supplement
part of its older package.

Fresh installation receipt
`250c5694108f621cab582f3a269dc8f09f8a140d14bb9d77ac95f7ab9857041e`,
selection `a807316ad92f988876137e9692bb1c8bd2ff2d8bb154efb44d7fd40329f1fff5`
and separate provider supplement
`894d2319590b1fdb4ca720af8d31c0507f9d1909cf19bc00d5f71053d31be92e`
join all four installed tools. Both before/after status calls reobserve the same
receipt and copied bytes. The historical package, three-tool installation and
first preparatory four-tool installation remain intact.

The synthetic ordinary input is a two-second H.264/AAC MP4 with 48 display
160×90 frames, coded height 96 and six cropped rows. Native acceptance returns a
648,164-byte clean master and a 71,090-byte delivery. Current payloads, source,
configuration, native intent and completion join; completed status/resume preserve
the native state inventory. Named controls refuse a second run, wrong resume,
changed source/configuration/payload and missing source. Actual selected-child
SIGINT returns structured 130 without harness forced cleanup; later status
returns typed recovery 7 with no committed domain acceptance. Descendant shutdown
remains unestablished.

Earlier private observations are deliberately retained. The first positive run
exceeded a reused 16 MiB observer member limit because Aniflow retains its 41.5 MB
provider. Two earlier cancellation observations could not discover a child because
this environment omits Linux `task/*/children`; the ordinary native runs completed
successfully and no cancellation success was claimed. The final observer reuses
the existing inventory with finite music bounds and bounded PPid lookup before
matching the exact installed executable inode. No product repair was inferred.

This is an actual Linux x86-64 source-candidate observation, not an immutable
published Aniflow/Renderflow release, authenticated suite, supported-host matrix,
native Mac trial, descendant-containment qualification or human creative approval.
Parent issues retain their full release, recovery and human-review acceptance;
Flow #88 remains unchanged and last.
