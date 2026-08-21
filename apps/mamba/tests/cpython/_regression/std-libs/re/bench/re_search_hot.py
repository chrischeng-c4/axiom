"""Hot-loop bench for stdlib re: compiled pattern search.

Domain: stdlib
Feature: re
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop running a compiled regex search on string inputs —
monomorphic string regime, measures regex match throughput.
"""
# tier: compute

import re

_pat = re.compile(r"\d+")
_texts = [f"item{i}:value={i*7}" for i in range(100)]
_n = len(_texts)

ITERS = 500_000

acc = 0
for i in range(ITERS):
    _m = _pat.search(_texts[i % _n])
    acc ^= (len(_m.group()) if _m else 0) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"re_search: {ITERS}")
