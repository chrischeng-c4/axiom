"""Hot-loop bench for stdlib string: Template.substitute.

Domain: stdlib
Feature: string
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop substituting into a string.Template —
monomorphic string regime, measures template substitution throughput.
"""
# tier: compute

import string

_tmpl = string.Template("user_${id}_v${version}")

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _s = _tmpl.substitute(id=v, version=v + 1)
    acc ^= len(_s) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"template_sub: {ITERS}")
