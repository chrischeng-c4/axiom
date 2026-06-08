# Operational AssertionPass seed for gc threshold/stats surface and
# generation-specific collect. Surface: `gc.collect()` runs a full
# collection across all generations and returns the non-negative
# count of objects collected; `gc.collect(generation)` collects only
# the named generation (0, 1, or 2). `gc.get_threshold()` returns a
# 3-tuple `(threshold0, threshold1, threshold2)` giving the
# generation-promotion thresholds. `gc.get_count()` returns the running 3-tuple of per-
# generation allocation counts. `gc.get_stats()` returns a list of
# per-generation statistics dicts. `gc.enable()` / `gc.disable()` /
# `gc.isenabled()` form the toggle triple. The `DEBUG_*` integer
# constants (`DEBUG_STATS`, `DEBUG_COLLECTABLE`,
# `DEBUG_UNCOLLECTABLE`, `DEBUG_LEAK`) are integer bitmasks.
import gc
_ledger: list[int] = []

# Full collection is a no-throw operation
gc.collect()
_ledger.append(1)

# get_count returns a 3-tuple of per-generation counts
counts = gc.get_count()
assert isinstance(counts, tuple); _ledger.append(1)
assert len(counts) == 3; _ledger.append(1)

# get_threshold returns a 3-tuple
thresh = gc.get_threshold()
assert isinstance(thresh, tuple); _ledger.append(1)
assert len(thresh) == 3; _ledger.append(1)

# enable / disable / isenabled toggle pair
gc.disable()
assert gc.isenabled() == False; _ledger.append(1)
gc.enable()
assert gc.isenabled() == True; _ledger.append(1)

# Generation-specific collect returns an int count
n = gc.collect(0)
assert isinstance(n, int); _ledger.append(1)
n2 = gc.collect()
assert isinstance(n2, int); _ledger.append(1)

# get_stats returns a list of per-generation stats dicts
stats = gc.get_stats()
assert isinstance(stats, list); _ledger.append(1)

# DEBUG_* constants are integer bitmasks
assert isinstance(gc.DEBUG_STATS, int); _ledger.append(1)
assert isinstance(gc.DEBUG_COLLECTABLE, int); _ledger.append(1)
assert isinstance(gc.DEBUG_UNCOLLECTABLE, int); _ledger.append(1)
assert isinstance(gc.DEBUG_LEAK, int); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_gc_threshold_stats_ops {sum(_ledger)} asserts")
