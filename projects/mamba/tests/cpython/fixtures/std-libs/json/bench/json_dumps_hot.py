"""Hot-loop bench for stdlib json: json.dumps of simple dict.

Domain: stdlib
Feature: json
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop serializing a small dict to JSON string —
monomorphic dict regime, measures json.dumps throughput.
"""
# tier: compute

import json

ITERS = 500_000

_inputs = [{"id": i, "val": i * 7, "active": True} for i in range(100)]
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    _s = json.dumps(_inputs[i % _n])
    acc ^= len(_s) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"json_dumps: {ITERS}")
