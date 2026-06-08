# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: subset/superset semantics and rich-comparison dispatch."""

p, q, r = set("ab"), set("abcde"), set("def")

# Proper-subset (<) is strict; subset (<=) is reflexive.
assert p < q
assert p <= q
assert q <= q
assert not (q < q)
assert q > p
assert q >= p
assert q >= q

# Disjoint / overlapping-but-not-nested sets are unordered.
assert not (q < r)
assert not (q <= r)
assert not (q > r)
assert not (q >= r)

# issubset / issuperset accept any iterable, not only sets.
assert set("a").issubset("abc")
assert set("abc").issuperset("a")
assert not set("a").issubset("cbs")
assert not set("cbs").issuperset("a")
assert {1, 2}.issubset([1, 2, 3])
assert {1, 2, 3}.issuperset((1, 2))

# A set compared against a non-set delegates to the other object's
# reflected comparison dunder.
class Probe:
    def __gt__(self, other):
        self.gt = True
        return False
    def __lt__(self, other):
        self.lt = True
        return False
    def __ge__(self, other):
        self.ge = True
        return False
    def __le__(self, other):
        self.le = True
        return False

s = {1, 2, 3}
probe = Probe(); _ = s < probe;  assert probe.gt   # s < x  -> x.__gt__(s)
probe = Probe(); _ = s > probe;  assert probe.lt   # s > x  -> x.__lt__(s)
probe = Probe(); _ = s <= probe; assert probe.ge   # s <= x -> x.__ge__(s)
probe = Probe(); _ = s >= probe; assert probe.le   # s >= x -> x.__le__(s)

print("ordering_and_subset OK")
