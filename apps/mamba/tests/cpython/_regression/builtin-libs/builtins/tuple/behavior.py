"""Behavior contract for builtins.tuple.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: tuple() with no args returns ()
assert tuple() == (), f"tuple() = {tuple()!r}"

# Rule 2: tuple(iterable)
assert tuple([1, 2, 3]) == (1, 2, 3), f"tuple([1,2,3]) = {tuple([1,2,3])!r}"
assert tuple(range(3)) == (0, 1, 2), f"tuple(range(3)) = {tuple(range(3))!r}"
assert tuple("abc") == ("a", "b", "c"), f"tuple('abc') = {tuple('abc')!r}"

# Rule 3: indexing
t = (10, 20, 30)
assert t[0] == 10, f"t[0] = {t[0]!r}"
assert t[-1] == 30, f"t[-1] = {t[-1]!r}"

# Rule 4: slicing returns tuple
t = (0, 1, 2, 3, 4)
assert t[1:3] == (1, 2), f"t[1:3] = {t[1:3]!r}"
assert t[::-1] == (4, 3, 2, 1, 0), f"t[::-1] = {t[::-1]!r}"
assert type(t[1:3]) is tuple, f"slice type = {type(t[1:3]).__name__!r}"

# Rule 5: immutability
_raised = False
try:
    t = (1, 2, 3)
    t[0] = 99  # type: ignore[index]
except TypeError:
    _raised = True
assert _raised, "tuple[0] = 99 did not raise TypeError"

# Rule 6: count
t = (1, 2, 1, 3, 1)
assert t.count(1) == 3, f"count(1) = {t.count(1)!r}"
assert t.count(9) == 0, f"count(9) = {t.count(9)!r}"

# Rule 7: index
t = (10, 20, 30, 20)
assert t.index(20) == 1, f"index(20) = {t.index(20)!r}"
_raised = False
try:
    t.index(99)
except ValueError:
    _raised = True
assert _raised, "index(99) did not raise ValueError"

# Rule 8: concatenation / repetition
assert (1, 2) + (3, 4) == (1, 2, 3, 4), "concatenation failed"
assert (0,) * 3 == (0, 0, 0), "repetition failed"
assert type((1, 2) + (3,)) is tuple, "concatenation type"

# Rule 9: in / not in
t = (1, 2, 3)
assert 2 in t, "2 in t failed"
assert 9 not in t, "9 not in t failed"

# Rule 10: len
assert len((1, 2, 3)) == 3, f"len = {len((1,2,3))!r}"
assert len(()) == 0, f"len(()) = {len(())!r}"

# Rule 11: unpacking
a, b, c = (1, 2, 3)
assert a == 1 and b == 2 and c == 3, "unpacking failed"

# Rule 12: starred unpacking
first, *rest = (1, 2, 3, 4)
assert first == 1, f"first = {first!r}"
assert rest == [2, 3, 4], f"rest = {rest!r}"

# Rule 13: single-element tuple requires trailing comma
t = (42,)
assert len(t) == 1, f"single-element tuple len = {len(t)!r}"
assert t[0] == 42, f"single-element tuple value = {t[0]!r}"

# Rule 14: nested tuples
t = ((1, 2), (3, 4))
assert t[0][1] == 2, f"nested = {t[0][1]!r}"

print("behavior OK")
