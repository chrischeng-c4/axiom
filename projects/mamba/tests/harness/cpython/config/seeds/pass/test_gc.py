# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: gc — enable / disable / isenabled state transitions, collect()
# returning an int, get_count() and get_threshold() returning 3-tuples,
# get_objects() returning a list. The gc.get_objects() list is a stub
# returning [] on mamba today (the runtime does not yet enumerate every
# tracked object), but its shape is stable; this seed asserts shape, not
# population.
import gc

_ledger: list[int] = []

# isenabled returns truthy after a fresh import (default-on)
assert gc.isenabled(), "gc.isenabled() is True by default after import"
_ledger.append(1)

# gc.disable() flips isenabled to falsy
gc.disable()
assert not gc.isenabled(), "gc.isenabled() is False after gc.disable()"
_ledger.append(1)

# gc.enable() flips it back to truthy
gc.enable()
assert gc.isenabled(), "gc.isenabled() is True after gc.enable()"
_ledger.append(1)

# gc.collect() returns an int (count of unreachable objects collected)
_n = gc.collect()
assert isinstance(_n, int), "gc.collect() returns an int"
_ledger.append(1)

assert _n >= 0, "gc.collect() returns a non-negative int"
_ledger.append(1)

# get_count() returns a 3-tuple of generation counters
_c = gc.get_count()
assert isinstance(_c, tuple), "gc.get_count() returns a tuple"
_ledger.append(1)

assert len(_c) == 3, f"gc.get_count() tuple has 3 entries, got {len(_c)}"
_ledger.append(1)

# get_threshold() returns a 3-tuple of generation thresholds
_t = gc.get_threshold()
assert isinstance(_t, tuple), "gc.get_threshold() returns a tuple"
_ledger.append(1)

assert len(_t) == 3, f"gc.get_threshold() tuple has 3 entries, got {len(_t)}"
_ledger.append(1)

# The generation-0 threshold is positive (gc actually fires)
assert _t[0] > 0, f"gc.get_threshold()[0] > 0, got {_t[0]}"
_ledger.append(1)

# get_objects() returns a list (may be a stub returning []; shape only)
_objs = gc.get_objects()
assert isinstance(_objs, list), "gc.get_objects() returns a list"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_gc {sum(_ledger)} asserts")
