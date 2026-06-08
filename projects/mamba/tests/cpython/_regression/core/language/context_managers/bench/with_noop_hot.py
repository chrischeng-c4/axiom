"""Hot-loop bench for language context managers: with statement overhead.

Domain: language
Feature: context_managers
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop entering a no-op context manager —
monomorphic on user-defined CM instance, measures with overhead.
"""
# tier: compute


ITERS = 500_000

class _Noop:
    def __enter__(self) -> "_Noop":
        return self
    def __exit__(self, *args) -> bool:
        return False

_cm = _Noop()

acc = 0
for i in range(ITERS):
    with _cm:
        acc ^= i & 0xFF

# Stdout sink — byte-equal across runtimes.
print(f"with_noop: {ITERS}")
