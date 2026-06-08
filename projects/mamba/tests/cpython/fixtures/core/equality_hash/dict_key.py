# mamba-xfail: classes that define __eq__ without __hash__ are still
# hashable on mamba (no implicit __hash__ = None), so the "unhashable"
# clause prints `False` instead of `True`. Identity/equality and
# collision clauses (Key, Colliding) already pass on mamba; the unhashable
# clause is the runtime gap that gates the xfail.
#
# Equality and hashing for dictionary keys — #2803.
#
# Covers user-defined __eq__ / __hash__ as it interacts with dict
# insertion, lookup, and collision handling. Includes the standard
# "defining __eq__ without __hash__ -> instances are unhashable"
# behavior; the unhashable probe is robust to brittle error messages
# (any TypeError counts as the expected outcome).
#
# Every print line is tagged with `[eq-hash]` so failure output names
# the semantic area.

# 1. Stable identity: two Key(1) instances should compare equal AND
#    share the same hash, so the second insertion overwrites the first.
class Key:
    def __init__(self, v):
        self.v = v
    def __eq__(self, other):
        return isinstance(other, Key) and self.v == other.v
    def __hash__(self):
        return hash(("Key", self.v))
    def __repr__(self):
        return "Key(" + str(self.v) + ")"

d = {}
d[Key(1)] = "first"
d[Key(1)] = "second"          # same key by __eq__/__hash__ → overwrites.
d[Key(2)] = "two"
print("len(d)=", len(d), "[eq-hash]")
print("d[Key(1)]=", d[Key(1)], "[eq-hash]")
print("d[Key(2)]=", d[Key(2)], "[eq-hash]")
print("Key(1) in d=", Key(1) in d, "[eq-hash]")
print("Key(3) in d=", Key(3) in d, "[eq-hash]")

# 2. Deterministic collision: Colliding(*) all hash to 0, so dict must
#    fall back to __eq__ to disambiguate within the same bucket.
class Colliding:
    def __init__(self, v):
        self.v = v
    def __eq__(self, other):
        return isinstance(other, Colliding) and self.v == other.v
    def __hash__(self):
        return 0          # force collisions for every instance.
    def __repr__(self):
        return "Coll(" + str(self.v) + ")"

c = {}
c[Colliding(1)] = "a"
c[Colliding(2)] = "b"
c[Colliding(3)] = "c"
print("len(c)=", len(c), "[eq-hash: collision]")
print("c[Coll(1)]=", c[Colliding(1)], "[eq-hash: collision]")
print("c[Coll(2)]=", c[Colliding(2)], "[eq-hash: collision]")
print("c[Coll(3)]=", c[Colliding(3)], "[eq-hash: collision]")

# Overwrite under collision: same v -> same eq -> overwrite slot.
c[Colliding(2)] = "B"
print("after overwrite c[Coll(2)]=", c[Colliding(2)], "[eq-hash: collision]")
print("len(c)=", len(c), "[eq-hash: collision]")

# 3. Defining __eq__ without __hash__ -> instances are unhashable.
class EqOnly:
    def __init__(self, v):
        self.v = v
    def __eq__(self, other):
        return isinstance(other, EqOnly) and self.v == other.v

raised = False
try:
    d2 = {}
    d2[EqOnly(7)] = "x"
except TypeError:
    raised = True
print("EqOnly unhashable raised=", raised, "[eq-hash: unhashable]")
