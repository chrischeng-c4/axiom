# Regression: dict(x) must accept generators, user iterables, and dicts.
# Prior to the fix, dispatch_dict ignored its argument and always returned
# an empty dict for any non-list/non-tuple input.

# dict(other_dict) — shallow copy
d1 = {"a": 1, "b": 2}
d2 = dict(d1)
print(d2)
print(d1 is d2)

# dict of list-pairs (regression: still works)
print(dict([("a", 1), ("b", 2)]))

# dict of tuple-pairs
print(dict((("x", 10), ("y", 20))))

# dict from a generator
def gen():
    yield ("p", 1)
    yield ("q", 2)
print(dict(gen()))

# dict from a user-defined iterable
class KV:
    def __iter__(self):
        yield ("m", 100)
        yield ("n", 200)
print(dict(KV()))

# Duplicate keys — last value wins
print(dict([("a", 1), ("b", 2), ("a", 3)]))

# Empty iterables
print(dict([]))
print(dict(()))
def empty():
    if False:
        yield
print(dict(empty()))
