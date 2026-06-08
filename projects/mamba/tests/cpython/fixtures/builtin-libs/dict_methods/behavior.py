# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: behavior asserts for less-common dict semantics."""


# update() accepts a duck-typed mapping that exposes keys() + __getitem__.
class SimpleMapping:
    def __init__(self):
        self.d = {1: 1, 2: 2, 3: 3}

    def keys(self):
        return self.d.keys()

    def __getitem__(self, k):
        return self.d[k]


d = {}
d.update(SimpleMapping())
assert d == {1: 1, 2: 2, 3: 3}

# update() with no argument is a no-op.
d.update()
assert d == {1: 1, 2: 2, 3: 3}

# A self-referential dict reprs with the {...} ellipsis sentinel.
r = {}
r[1] = r
assert repr(r) == "{1: {...}}"

# repr of an empty / simple dict is exact.
assert repr({}) == "{}"
assert repr({1: 2}) == "{1: 2}"

# fromkeys called on a dict subclass returns an instance of that subclass.
class MyDict(dict):
    pass

md = MyDict.fromkeys("ab")
assert md == {"a": None, "b": None}
assert isinstance(md, MyDict)
assert isinstance(MyDict().fromkeys("a"), MyDict)

# fromkeys consumes an arbitrary iterable / generator and shares the default.
def gen():
    yield 1
    yield 2

assert dict.fromkeys(gen()) == {1: None, 2: None}
assert dict.fromkeys((4, 5), 0) == {4: 0, 5: 0}
assert dict.fromkeys([]) == {}

# Views are live windows: mutating the dict updates previously-taken views.
src = {"a": 1}
ks, vs, its = src.keys(), src.values(), src.items()
src["b"] = 2
assert set(ks) == {"a", "b"}
assert set(vs) == {1, 2}
assert set(its) == {("a", 1), ("b", 2)}
del src["a"]
assert set(ks) == {"b"}
assert len(ks) == 1

# setdefault returns None for a missing key with no default, and the inserted
# mutable can be built up across calls.
acc = {}
assert acc.setdefault("k") is None
acc.setdefault("list", []).append(3)
acc.setdefault("list", []).append(4)
assert acc["list"] == [3, 4]

print("behavior OK")
