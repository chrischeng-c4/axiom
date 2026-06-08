"""Hot-loop bench for `google.protobuf.message` /
`google.protobuf.descriptor` / `google.protobuf.json_format` /
`google.protobuf.text_format` module-attribute reads (#1513).

End-user scenario: gRPC and protobuf-using services re-resolve
`google.protobuf.message` (the base `Message` class shells),
`google.protobuf.descriptor` (the descriptor pool entry-points),
`google.protobuf.json_format` (MessageToJson / Parse helpers),
and `google.protobuf.text_format` (text-proto serializer) on
every encode / decode call site. Per-request RPC dispatch
re-resolves these names through the package's attribute table on
each call site. That per-call module-attribute quadruple-read is
the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are submodules attached to
`google.protobuf` via explicit `import google.protobuf.<name>`).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `google.protobuf`
module-attribute resolver, short-circuiting CPython's module-dict
probe chain for read-only sentinels.

Workload: 20_000 paired reads of `message`, `descriptor`,
`json_format`, and `text_format` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import google.protobuf as _p
# CPython: explicit submodule imports attach `message`, `descriptor`,
# `json_format`, `text_format` to the `google.protobuf` namespace.
# Mamba: the shim pre-registers all four as identity-stable dispatchers,
# so the dotted-import statements are no-ops on the mamba side.
try:
    import google.protobuf.message      # noqa: F401
    import google.protobuf.descriptor   # noqa: F401
    import google.protobuf.json_format  # noqa: F401
    import google.protobuf.text_format  # noqa: F401
except Exception:
    pass


_MESSAGE_BASELINE = _p.message
_DESCRIPTOR_BASELINE = _p.descriptor
_JSON_FORMAT_BASELINE = _p.json_format
_TEXT_FORMAT_BASELINE = _p.text_format

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _p.message
    b = _p.descriptor
    c = _p.json_format
    d = _p.text_format
    if (a is _MESSAGE_BASELINE
            and b is _DESCRIPTOR_BASELINE
            and c is _JSON_FORMAT_BASELINE
            and d is _TEXT_FORMAT_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"google.protobuf module-attribute read acc drift: acc={acc} expected={ITERS}"
print("protobuf_type_read_hot:", acc)
