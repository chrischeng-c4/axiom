"""Hot-loop bench for stdlib pathlib: Path join + str.

Domain: stdlib
Feature: pathlib
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop constructing paths via / operator and converting to str —
monomorphic string regime, measures Path join + str overhead.
"""
# tier: compute

from pathlib import Path

ITERS = 500_000

_bases = [f"dir{i}" for i in range(100)]
_files = [f"file{i}.txt" for i in range(100)]
_n = len(_bases)

_root = Path("/tmp")
acc = 0
for i in range(ITERS):
    _p = _root / _bases[i % _n] / _files[i % _n]
    acc ^= len(str(_p)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"path_join: {ITERS}")
