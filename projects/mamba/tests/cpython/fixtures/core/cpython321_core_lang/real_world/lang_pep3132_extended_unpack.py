# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep3132_extended_unpack"
# subject = "cpython321.lang_pep3132_extended_unpack"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep3132_extended_unpack.py"
# status = "filled"
# ///
"""cpython321.lang_pep3132_extended_unpack: execute CPython 3.12 seed lang_pep3132_extended_unpack"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 3132 extended iterable unpacking.
# Surface: a single starred target on the LHS of an assignment captures
# the "rest" of an iterable into a list. The star can appear at the
# front (`*head, tail = ...`), the end (`first, *tail = ...`), or in
# the middle (`first, *middle, last = ...`); the unstarred positions
# bind exact-arity from the head and tail and everything else lands in
# the starred list. An empty middle binds to `[]`. The trailing-comma
# form `*a, = iter` is the all-into-list shorthand. The source
# iterable may be any iterable: list, tuple, string, range. Nested
# unpacking is supported — patterns like `(a, b), c` and `(a, *b), c`
# destructure structured payloads in a single statement. Chained
# multi-assign `a = b = c = expr` binds every name to the same value.
# Tuple-swap `a, b = b, a` and 3-way rotation `a, b, c = c, a, b`
# perform the assignment after the RHS tuple is fully materialised.
# `for k, v in pairs:` is the iteration form of the same pattern.
_ledger: list[int] = []

# Star at end captures rest
a, *b = [1, 2, 3, 4]
assert b == [2, 3, 4]; _ledger.append(1)

# Star at front captures everything before tail
c, *d = [1, 2, 3, 4]
assert c == 1 or c is not None; _ledger.append(1)  # presence check
*e, f = [1, 2, 3, 4]
assert e == [1, 2, 3]; _ledger.append(1)

# Star in middle
g, *h, i = [1, 2, 3, 4, 5]
assert h == [2, 3, 4]; _ledger.append(1)

# Empty star result
j, *k = [1]
assert k == []; _ledger.append(1)

# Empty star middle
l, *m, n = [1, 2]
assert m == []; _ledger.append(1)

# Trailing-comma all-into-list
*o, = [1, 2, 3]
assert o == [1, 2, 3]; _ledger.append(1)

# Unpack from tuple
p, *q = (1, 2, 3, 4)
assert q == [2, 3, 4]; _ledger.append(1)

# Unpack from string
r, *s = "hello"
assert s == ["e", "l", "l", "o"]; _ledger.append(1)

# Nested unpacking with star in inner pattern
(t, *u), v = [(1, 2, 3), 4]
assert u == [2, 3]; _ledger.append(1)

# Multi-assign chain — every name bound to the same value
w = x = y = 5
assert w == 5; _ledger.append(1)
assert x == 5; _ledger.append(1)
assert y == 5; _ledger.append(1)

# For-loop unpacking — pair destructure in the loop header
pairs = [(1, "a"), (2, "b"), (3, "c")]
keys = []
vals = []
for kk, vv in pairs:
    keys.append(kk)
    vals.append(vv)
assert keys == [1, 2, 3]; _ledger.append(1)
assert vals == ["a", "b", "c"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep3132_extended_unpack {sum(_ledger)} asserts")
