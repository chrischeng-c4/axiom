"""Hot-loop bench for language exceptions: try/except with no raise.

Domain: language
Feature: exceptions
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop executing try/except where no exception
is raised — measures overhead of exception guard on the happy path.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    try:
        v = _inputs[i % _n]
        acc ^= v
    except IndexError:
        acc ^= 0xFF

# Stdout sink — byte-equal across runtimes.
print(f"try_except: {ITERS}")
