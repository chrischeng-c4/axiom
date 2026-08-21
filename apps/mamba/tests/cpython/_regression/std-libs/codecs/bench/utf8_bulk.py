"""Bulk-bytes hot-loop bench for `codecs` (Task #34 — Wave-1 收尾).

End-user scenario: bulk UTF-8 encode/decode of a 1MB Unicode string —
the canonical bulk-bytes tier:compute path. Each iteration crosses the
FFI boundary exactly twice (`encode` + `decode`), so per-element
dispatch overhead amortizes over the whole MB-scale buffer and the
floor for `tier:compute` is wall >=10x.

#2096 subset A (bulk-bytes-materialization): the hot path produces a
multi-MB `bytes` object per call, so per-object header overhead
amortizes inversely with payload size. Expected memory band: 0.5-0.9x
of CPython (ship with carve-out (iii) per [[feedback_mamba_perf_is_the_product]]).

Hoist convention (per #2097): module-level attributes are hoisted to
locals BEFORE the hot loop. Without hoisting, mamba's module-attr
lookup at the call site is ~5x slower than the hoisted form.

#2105 avoidance: no `assert` between the hot-loop call and the next
statement that depends on it. The post-loop `print` of `acc_len`

# tier: compute
"""

import codecs

# Hoist module-level attributes outside the loop (#2097).
encode = codecs.encode
decode = codecs.decode

# Build a ~1MB UTF-8 input string. Mix of ASCII + multi-byte chars so
# the codec actually does Unicode work, not just memcpy. Repeated
# fragment, so it's compressible enough that the bytes layout makes
# the memory delta observable (#2096 subset A).
FRAGMENT = "mamba codecs bulk encode test: hello world 1234567890 " \
           "中文測試 éèêë " \
           "emoji-region: ☃☀⚡ "
# ~150 bytes per fragment * ~7000 = ~1MB.
PAYLOAD = FRAGMENT * 7000

ITERS = 100

acc_len = 0
for _ in range(ITERS):
    b = encode(PAYLOAD, "utf-8")
    s = decode(b, "utf-8")
    acc_len += len(s)
print("codecs_utf8_bulk:", acc_len)
