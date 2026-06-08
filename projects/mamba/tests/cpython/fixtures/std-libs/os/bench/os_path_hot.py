"""Hot-loop bench for stdlib os: os.path.join + basename.

Domain: stdlib
Feature: os
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling os.path.join and os.path.basename —
monomorphic string regime, measures path manipulation throughput.
"""
# tier: compute

import os.path

ITERS = 500_000

_dirs = [f"dir{i}" for i in range(100)]
_files = [f"file{i}.txt" for i in range(100)]
_n = len(_dirs)

_join = os.path.join
_basename = os.path.basename

acc = 0
for i in range(ITERS):
    _p = _join("/tmp", _dirs[i % _n], _files[i % _n])
    acc ^= len(_basename(_p)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"os_path: {ITERS}")
