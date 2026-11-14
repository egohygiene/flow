# Installation readback — November 13, 2026

Relates to #71. Exact installation/offline-bundle membership now counts every
visited entry, including within flat directories, and checks cancellation at
each directory/entry. The 512-entry and four-level limits apply across the
whole tree. Oversized or interrupted readback preserves unknown material and
cannot reach activation/removal through a falsely complete observation.

Two regressions are authored, uncompiled/unrun. No tests/builds/dependencies,
validators, installers, product/native/provider/media or CI ran. This source
checkpoint does not qualify installation, rollback, uninstall, a distribution
channel, a provider profile or any supported-host matrix cell. #71 stays open;
Flow #88 stays last and untouched.
