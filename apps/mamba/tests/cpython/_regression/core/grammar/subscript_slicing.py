# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: subscription and slicing grammar (CPython 3.12 oracle).

Distilled from CPython GrammarTests selectors + TestSpecifics subscripts:
slice syntax produces slice() objects, tuple subscripts produce tuples,
Ellipsis is a valid subscript, and the get/set/del/aug-assign dunders fire.
"""

# Basic indexing and the full family of slice forms.
s = "01234"
assert s[0] == "0"
assert s[-1] == "4"
assert s[0:5] == "01234"
assert s[:5] == "01234"
assert s[2:] == "234"
assert s[:] == "01234"
assert s[-4:-3] == "1"
assert s[::2] == "024"
assert s[::-1] == "43210"
print("slices: ok")

# A multi-element subscript is a tuple key; a trailing comma makes a 1-tuple.
d = {}
d[1] = "one"
d[1,] = "one_tuple"
d[1, 2] = "two"
d[1, 2, 3] = "three"
assert d[1] == "one"
assert d[(1,)] == "one_tuple"
assert d[1, 2] == "two"
assert sorted(d, key=lambda k: (type(k).__name__, k)) == [1, (1,), (1, 2), (1, 2, 3)]
print("tuple_subscript: ok")

# Slice syntax inside a subscript builds slice objects; bare colon -> slice(None).
assert s[1:4:2] == "13"


class Probe:
    def __init__(self):
        self.last = None
        self.store = {}

    def __getitem__(self, key):
        self.last = key
        return self.store.get(repr(key), 0)

    def __setitem__(self, key, value):
        self.last = key
        self.store[repr(key)] = value

    def __delitem__(self, key):
        self.last = key
        self.store.pop(repr(key), None)


p = Probe()
_ = p[1:2:3]
assert p.last == slice(1, 2, 3)
_ = p[:]
assert p.last == slice(None, None, None)
_ = p[1:2, 3:4]
assert p.last == (slice(1, 2), slice(3, 4))
_ = p[...]
assert p.last is Ellipsis
print("slice_objects: ok")

# Augmented assignment through a subscript does get-then-set.
p[5] = 10
p[5] += 1
assert p[5] == 11
del p[5]
assert p[5] == 0
print("subscript_slicing OK")
