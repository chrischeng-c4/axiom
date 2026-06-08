# Operational AssertionPass seed for tuple-swap and multi-target
# assignment patterns. Surface: basic two-name destructuring; one-line
# tuple swap (the canonical Pythonism — no temporary needed); three-
# or-more rotation; chained `a = b = c = expr` assignment with the
# same RHS reference (incl. shared mutability when RHS is a list);
# PEP 3132 starred unpacking — `first, *rest = ...`, `*init, last`,
# `head, *mid, tail`, and the degenerate-empty-middle case;
# single-element `e, = (42,)` trailing-comma form; nested tuple
# destructuring; multi-return functions with `tuple[int, int]`
# annotations (avoiding the int-return-identity quirk); starred
# unpacking from a function call; destructuring from tuple vs. list
# RHS forms.
_ledger: list[int] = []

# Basic two-name destructuring
a, b = 1, 2
assert a == 1; _ledger.append(1)
assert b == 2; _ledger.append(1)

# The canonical one-line swap — no temporary needed
a, b = b, a
assert a == 2; _ledger.append(1)
assert b == 1; _ledger.append(1)

# Three-way rotation
x, y, z = 1, 2, 3
x, y, z = z, x, y
assert x == 3; _ledger.append(1)
assert y == 1; _ledger.append(1)
assert z == 2; _ledger.append(1)

# Full reversal via three-target swap
p1, p2, p3 = 1, 2, 3
p1, p2, p3 = p3, p2, p1
assert p1 == 3; _ledger.append(1)
assert p2 == 2; _ledger.append(1)
assert p3 == 1; _ledger.append(1)

# Chained `a = b = c = expr` — all three names bind to the same value
ca = cb = cc = 5
assert ca == 5; _ledger.append(1)
assert cb == 5; _ledger.append(1)
assert cc == 5; _ledger.append(1)

# Chained assign with a mutable RHS — both names refer to the SAME
# object, so mutating one is visible through the other
p = q = [1, 2]
p.append(3)
assert q == [1, 2, 3]; _ledger.append(1)
assert p is q; _ledger.append(1)

# PEP 3132 — starred unpacking on the LHS
first, *rest = [1, 2, 3, 4]
assert first == 1; _ledger.append(1)
assert rest == [2, 3, 4]; _ledger.append(1)
# The starred name always binds to a list (even on a tuple/iterable RHS)
assert type(rest).__name__ == "list"; _ledger.append(1)

# Starred at the END — `*init, last`
*init, last = [1, 2, 3, 4]
assert init == [1, 2, 3]; _ledger.append(1)
assert last == 4; _ledger.append(1)

# Starred in the MIDDLE — `head, *mid, tail`
head, *mid, tail = [1, 2, 3, 4, 5]
assert head == 1; _ledger.append(1)
assert mid == [2, 3, 4]; _ledger.append(1)
assert tail == 5; _ledger.append(1)

# Starred with empty middle (RHS has exactly enough for non-starred names)
first2, *rest2 = [1]
assert first2 == 1; _ledger.append(1)
assert rest2 == []; _ledger.append(1)

# Nested destructuring — `(a, b), c = (1, 2), 3`
(a1, b1), c1 = (1, 2), 3
assert a1 == 1; _ledger.append(1)
assert b1 == 2; _ledger.append(1)
assert c1 == 3; _ledger.append(1)

# Single-element trailing-comma form — `e, = (42,)`
e, = (42,)
assert e == 42; _ledger.append(1)

# Multi-return functions with typed tuple return — avoids the int-
# return-identity quirk by destructuring directly at the call site
def two() -> tuple[int, int]:
    return (10, 20)
r1, r2 = two()
assert r1 == 10; _ledger.append(1)
assert r2 == 20; _ledger.append(1)

# Swap helper — function returning a tuple
def swap_fn(a: int, b: int) -> tuple[int, int]:
    return b, a
sx, sy = swap_fn(1, 2)
assert sx == 2; _ledger.append(1)
assert sy == 1; _ledger.append(1)

# Three-value unpack from a function call
def make() -> tuple[int, int, int]:
    return 100, 200, 300
m1, m2, m3 = make()
assert m1 == 100; _ledger.append(1)
assert m2 == 200; _ledger.append(1)
assert m3 == 300; _ledger.append(1)

# Starred unpack from a function call that returns a list
def all_three() -> list[int]:
    return [1, 2, 3]
f1, *fr = all_three()
assert f1 == 1; _ledger.append(1)
assert fr == [2, 3]; _ledger.append(1)

# Destructure from tuple RHS
a2, b2 = (5, 6)
assert a2 == 5; _ledger.append(1)
assert b2 == 6; _ledger.append(1)

# Destructure from list RHS (Python's destructuring is iterable-driven)
c2, d2 = [7, 8]
assert c2 == 7; _ledger.append(1)
assert d2 == 8; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_swap_multi_assign {sum(_ledger)} asserts")
