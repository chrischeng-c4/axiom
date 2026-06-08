"""Hot-loop bench for builtins.pow: pow(base, exp) on integers.

Domain: builtins
Feature: pow
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling pow(base, exp) on small positive
integers — monomorphic on int inputs.
"""
# tier: compute


ITERS = 500_000

_bases = list(range(1, 11))   # 1..10
_exps  = list(range(0, 6))    # 0..5
_n_b = len(_bases)
_n_e = len(_exps)

acc = 0
for i in range(ITERS):
    base = _bases[i % _n_b]
    exp = _exps[i % _n_e]
    result = pow(base, exp)
    acc = (acc + result) % 1_000_000

# Stdout sink — byte-equal across runtimes.
print(f"pow_int: {ITERS}")
