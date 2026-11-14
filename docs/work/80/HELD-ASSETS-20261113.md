# Local AI asset reads — November 13, 2026

Relates to #80. The byte-preparation owner now opens selected assets with
non-following/nonblocking flags on Unix and checks the held descriptor against
the selected path before and after bounded hashing. Singly linked regular-file
identity, extent, modification time and ctime must agree. A matching digest and
mtime cannot hide replacement of the selected path. Platforms without this
identity observation remain unavailable rather than prepared.

Two focused regressions are authored, uncompiled/unrun. No tests/builds,
dependencies, validators, product/provider/model/native execution or CI ran.
No model/service was contacted, acquired or launched. Byte preparation still
does not establish native model identity, memory/offline enforcement, license
review or invocation authority. #80 remains open for its retained integration
and qualification; Flow #88 remains last and untouched.
