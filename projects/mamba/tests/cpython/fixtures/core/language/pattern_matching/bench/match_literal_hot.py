"""Hot-loop bench for language pattern matching: literal match dispatch.

Domain: language
Feature: pattern_matching
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop dispatching on small integer literals with
match/case — monomorphic int regime, measures match-statement dispatch overhead.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n] % 5
    match v:
        case 0:
            acc ^= 1
        case 1:
            acc ^= 2
        case 2:
            acc ^= 4
        case 3:
            acc ^= 8
        case _:
            acc ^= 16

# Stdout sink — byte-equal across runtimes.
print(f"match_literal: {ITERS}")
