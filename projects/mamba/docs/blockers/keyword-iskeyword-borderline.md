# keyword/iskeyword_hot — borderline wall 0.88×

**Workload**: `keyword.iskeyword(name)` membership check over a hot loop
(post-#2100-bundle sweep, 2026-05-15, commit `015b820a2`).

**Ratio**: wall 0.88× CPython (FAIL by 0.12×). Set-membership should
beat CPython easily; suspect startup-dominated or `_kwlist` frozenset
dispatch overhead.

**Status**: needs scout. Not blocking; not #2100-bound.
