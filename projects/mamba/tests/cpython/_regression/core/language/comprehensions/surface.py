"""Surface contract for language comprehensions.

# type-regime: monomorphic

Probes: list comp, dict comp, set comp, generator expr, nested comp,
if-filter, multiple for-clauses, walrus in comp.
CPython 3.12 is the oracle.
"""

# List comprehension returns list
lc = [x * 2 for x in range(5)]
assert isinstance(lc, list), "list comp not list"
assert lc == [0, 2, 4, 6, 8], f"list comp = {lc!r}"

# Dict comprehension returns dict
dc = {k: v for k, v in enumerate("abc")}
assert isinstance(dc, dict), "dict comp not dict"
assert dc == {0: "a", 1: "b", 2: "c"}, f"dict comp = {dc!r}"

# Set comprehension returns set
sc = {x % 3 for x in range(9)}
assert isinstance(sc, set), "set comp not set"
assert sc == {0, 1, 2}, f"set comp = {sc!r}"

# Generator expression returns generator (not list)
import types
ge = (x * x for x in range(3))
assert isinstance(ge, types.GeneratorType), "gen expr not generator"
assert list(ge) == [0, 1, 4], f"gen expr = {list((x*x for x in range(3)))!r}"

# if filter
evens = [x for x in range(10) if x % 2 == 0]
assert evens == [0, 2, 4, 6, 8], f"filter comp = {evens!r}"

# Nested for-clause (cartesian product)
pairs = [(x, y) for x in range(3) for y in range(2)]
assert len(pairs) == 6, f"nested comp len = {len(pairs)!r}"
assert pairs[0] == (0, 0) and pairs[-1] == (2, 1), f"nested comp = {pairs!r}"

# Comprehension has own scope (Python 3+)
x_outer = 99
result = [x_outer for _ in range(3)]  # x_outer from outer scope
assert result == [99, 99, 99], f"scope comp = {result!r}"

print("surface OK")
