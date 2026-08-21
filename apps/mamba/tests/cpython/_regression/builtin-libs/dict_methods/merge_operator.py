# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: PEP 584 | and |= merge operators (incl. dunder edge cases)."""

a = {0: 0, 1: 1, 2: 1}
b = {1: 1, 2: 2, 3: 3}

# | returns a new dict; right operand wins on key collisions.
assert a | b == {0: 0, 1: 1, 2: 2, 3: 3}
assert b | a == {1: 1, 2: 1, 3: 3, 0: 0}
assert a == {0: 0, 1: 1, 2: 1}    # | does not mutate either operand

# |= mutates in place, accepting another dict.
c = a.copy()
c |= b
assert c == {0: 0, 1: 1, 2: 2, 3: 3}

# |= also accepts any iterable of key/value pairs.
c = a.copy()
c |= [(1, 1), (2, 2), (3, 3)]
assert c == {0: 0, 1: 1, 2: 2, 3: 3}

# __or__ returns NotImplemented for non-mapping right operands so that
# Python can fall back / raise TypeError at the operator level.
assert a.__or__(None) is NotImplemented
assert a.__or__(()) is NotImplemented
assert a.__or__("BAD") is NotImplemented

# __ior__ is more permissive: it consumes iterables of pairs in place.
d = a.copy()
assert d.__ior__(()) == {0: 0, 1: 1, 2: 1}      # empty iterable: no change
assert d.__ior__("") == {0: 0, 1: 1, 2: 1}      # empty string: no change

# __ior__ rejects None (TypeError) and malformed strings (ValueError).
try:
    a.copy().__ior__(None)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

try:
    a.copy().__ior__("BAD")
    raise AssertionError("expected ValueError")
except ValueError:
    pass

print("merge_operator OK")
