# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: exceptions from __hash__/__eq__/__repr__ propagate unswallowed."""


class Boom(Exception):
    pass


# A key whose __eq__ raises propagates on every lookup-style operation once a
# hash collision forces an equality check.
class BadEq:
    def __hash__(self):
        return 7

    def __eq__(self, other):
        if isinstance(other, BadEq):
            raise Boom
        return NotImplemented


d = {}
x1 = BadEq()
d[x1] = 1                       # first insert: no equality check needed
x2 = BadEq()                    # same hash bucket -> equality check vs x1
for label, op in [
    ("setitem", lambda: d.__setitem__(x2, 2)),
    ("getitem", lambda: d[x2]),
    ("contains", lambda: x2 in d),
    ("get", lambda: d.get(x2)),
    ("setdefault", lambda: d.setdefault(x2, 42)),
    ("pop", lambda: d.pop(x2)),
    ("update", lambda: {}.update({x1: 0}) or d.update({x2: 9})),
]:
    try:
        op()
        raise AssertionError(f"{label}: expected Boom")
    except Boom:
        pass


# A key whose __hash__ raises propagates through hashing operations.
class BadHash:
    fail = False

    def __hash__(self):
        if self.fail:
            raise Boom
        return 42


h = BadHash()
d2 = {h: "v"}
h.fail = True
for label, op in [
    ("getitem", lambda: d2[h]),
    ("setdefault", lambda: d2.setdefault(h, [])),
    ("pop", lambda: d2.pop(h)),
]:
    try:
        op()
        raise AssertionError(f"{label}: expected Boom")
    except Boom:
        pass


# An exception from a value's __repr__ propagates out of repr(dict).
class BadRepr:
    def __repr__(self):
        raise Boom


try:
    repr({1: BadRepr()})
    raise AssertionError("repr: expected Boom")
except Boom:
    pass


# Dict equality that triggers a key's raising __eq__ propagates.
try:
    {BadEq(): 1} == {BadEq(): 1}
    raise AssertionError("dict_eq: expected Boom")
except Boom:
    pass

print("key_exceptions_propagate OK")
