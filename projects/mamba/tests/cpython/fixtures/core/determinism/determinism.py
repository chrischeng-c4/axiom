# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/determinism: deterministic-execution properties (CPython 3.12 oracle)."""

# (1) dict preserves insertion order; iteration order is reproducible
# within a run. Build the same mapping two ways and from two iterations.
keys = ["zeta", "alpha", "mid", "9", "beta", "alpha"]  # dup re-seats value, not slot
d1 = {}
for i, k in enumerate(keys):
    d1[k] = i
d2 = dict(d1)
assert list(d1) == list(d1), "dict iteration must be stable across calls"
assert list(d1) == list(d2), "dict copy must preserve key order"
assert list(d1) == ["zeta", "alpha", "mid", "9", "beta"], "insertion order, dup keeps slot"
assert list(d1.items()) == list(d2.items()), "items() order stable"
print("dict_order_guard:", list(d1) == list(d2))

# set: iteration order is reproducible within a run (same object, repeated
# iteration yields the same sequence; a copy yields the same sequence).
s = set()
for n in [3, 1, 4, 1, 5, 9, 2, 6]:
    s.add(n)
order_a = list(s)
order_b = list(s)
order_copy = list(set(s))
assert order_a == order_b, "set iteration must be stable within a run"
assert order_a == order_copy, "set copy iteration must match"
assert sorted(order_a) == [1, 2, 3, 4, 5, 6, 9], "set membership content"
print("set_order_guard:", order_a == order_b == order_copy)

# (2) hash() of small ints and of a given str is stable WITHIN this process.
# Note: PYTHONHASHSEED randomizes str hashing ACROSS processes, so we do not
# assert any specific value or cross-process equality -- only within-run
# stability of repeated hash() calls on equal objects.
for n in range(-5, 33):
    assert hash(n) == hash(n), "int hash unstable within run"
    assert hash(n) == hash(int(str(n))), "equal ints must hash equal"
assert hash(0) == 0 and hash(1) == 1, "small-int hash is the value"
assert hash(-1) == hash(-2) - 1 or hash(-1) == -2, "hash(-1) is the -2 special case"
sample = "deterministic-execution"
assert hash(sample) == hash(sample), "str hash unstable within run"
assert hash(sample) == hash("deterministic-" + "execution"), "equal strs hash equal"
assert hash("") == 0, "empty-str hash is 0 within and across runs"
print("hash_within_run_guard:", hash(sample) == hash(sample))

# Equal objects of compatible types share a hash within the run.
assert hash(1) == hash(1.0) == hash(True), "1 == 1.0 == True implies equal hash"
print("hash_equal_objects_guard:", hash(1) == hash(1.0) == hash(True))

# (3) sorted() is a total order on homogeneous input: reflexive (idempotent),
# antisymmetric/transitive (a valid permutation), and order-independent of the
# input permutation.
import random

base = [37, 4, 19, 4, 0, -8, 256, 19, 11, -8, 99]
ref = sorted(base)
assert ref == sorted(ref), "sorted is idempotent (already sorted stays put)"
assert all(ref[i] <= ref[i + 1] for i in range(len(ref) - 1)), "non-decreasing"
assert sorted(ref, reverse=True) == ref[::-1], "reverse is the exact mirror"

rng = random.Random(0)  # seeded -> the fixture itself stays deterministic
for _ in range(50):
    perm = base[:]
    rng.shuffle(perm)
    assert sorted(perm) == ref, "sort result independent of input permutation"
print("total_order_guard:", sorted(base) == ref)

# String input: total order is lexicographic and permutation-independent.
words = ["pear", "apple", "fig", "apple", "Apple", "banana"]
wref = sorted(words)
assert all(wref[i] <= wref[i + 1] for i in range(len(wref) - 1)), "str non-decreasing"
assert sorted(reversed(words)) == sorted(words), "str sort permutation-independent"
print("str_total_order_guard:", sorted(words) == wref)

print("determinism OK")
