# Operational AssertionPass seed for `gc` cycle-collector surface.
# Surface: gc.enable / gc.disable / gc.isenabled toggle pair;
# gc.get_count returns a 3-tuple of per-generation counts;
# gc.collect returns a non-negative count of collected objects.
# Companion to stub/test_gc.py — vendored unittest seed.
import gc
_ledger: list[int] = []
# Default state — GC is enabled at interpreter startup
assert gc.isenabled(); _ledger.append(1)
# Toggle off / on round-trip
gc.disable()
assert not gc.isenabled(); _ledger.append(1)
gc.enable()
assert gc.isenabled(); _ledger.append(1)
# get_count returns a 3-tuple of per-generation counts
counts = gc.get_count()
assert len(counts) == 3; _ledger.append(1)
# Each generation count is non-negative
assert counts[0] >= 0; _ledger.append(1)
assert counts[1] >= 0; _ledger.append(1)
assert counts[2] >= 0; _ledger.append(1)
# collect returns a non-negative collected-object count
n = gc.collect()
assert n >= 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_gc_ops {sum(_ledger)} asserts")
