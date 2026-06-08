"""Hot-loop bench for builtins.callable: callable() on mixed objects.

Domain: builtins
Feature: callable
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling callable() on built-in functions
— monomorphic on builtin_function_or_method inputs.
"""
# tier: compute


ITERS = 500_000

_targets = [len, print, int, str, list, dict, set, tuple, range, type]
_n = len(_targets)

acc = 0
for i in range(ITERS):
    acc ^= int(callable(_targets[i % _n]))

# Stdout sink — byte-equal across runtimes.
print(f"callable_check: {ITERS}")
