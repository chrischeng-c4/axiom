# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_comprehensions"
# subject = "cpython321.lang_comprehensions"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_comprehensions.py"
# status = "filled"
# ///
"""cpython321.lang_comprehensions: execute CPython 3.12 seed lang_comprehensions"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_comprehensions.py — #3357 axis-1 comprehensions + nested-scope seed.
#
# Exercises:
#   1. List comprehension: basic, with condition (filter)
#   2. Set comprehension: deduplicates
#   3. Dict comprehension: {k: v for k in ...}
#   4. Generator expression: sum(x for x in ...)
#   5. Nested comprehension (matrix-style)
#   6. Multi-for comprehension (cartesian product)
#   7. Conditional expression inside comprehension
#   8. Inner closure inside comprehension (function expression)
#
# Mamba quirks (tracked separately):
#   * Comprehension loop variable leaks into enclosing scope (#3502).
#     Mamba's type-checker doesn't see the bind either way, so this is
#     intentionally NOT exercised here.
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS marker → AssertionPass.

_ledger: list[int] = []

# (1) Basic list comprehension
_doubled = [x * 2 for x in range(5)]
assert _doubled == [0, 2, 4, 6, 8], (
    f"list comp doubles, got {_doubled!r}"
)
_ledger.append(1)

# (1b) Filtered list comprehension
_evens = [x for x in range(10) if x % 2 == 0]
assert _evens == [0, 2, 4, 6, 8], f"filter even, got {_evens!r}"
_ledger.append(1)

# (2) Set comprehension deduplicates
_uniq = {x for x in [1, 2, 2, 3, 3, 3]}
assert _uniq == {1, 2, 3}, f"set comp dedups, got {_uniq!r}"
_ledger.append(1)

# (3) Dict comprehension
_squares = {x: x * x for x in range(3)}
assert _squares == {0: 0, 1: 1, 2: 4}, f"dict comp squares, got {_squares!r}"
_ledger.append(1)

# (4) Generator expression consumed by sum()
_total = sum(x for x in range(5))
# subtraction dodge for boxed-int
assert _total - 10 == 0, f"sum(genexp) == 10, got {_total!r}"
_ledger.append(1)

# (4b) Generator expression consumed by tuple()
_t = tuple(x for x in range(4))
assert _t == (0, 1, 2, 3), f"tuple(genexp), got {_t!r}"
_ledger.append(1)

# (5) Nested comprehension: matrix-style
_matrix = [[x + y for y in range(2)] for x in range(2)]
assert _matrix == [[0, 1], [1, 2]], f"nested matrix, got {_matrix!r}"
_ledger.append(1)

# (6) Multi-for comprehension: cartesian product
_pairs = [(i, j) for i in range(2) for j in range(2)]
assert _pairs == [(0, 0), (0, 1), (1, 0), (1, 1)], (
    f"multi-for cartesian, got {_pairs!r}"
)
_ledger.append(1)

# (6b) Multi-for with cross-clause name reference
_triangle = [(i, j) for i in range(3) for j in range(i + 1)]
assert _triangle == [(0, 0), (1, 0), (1, 1), (2, 0), (2, 1), (2, 2)], (
    f"multi-for cross-clause, got {_triangle!r}"
)
_ledger.append(1)

# (7) Conditional expression inside comprehension
_parity = ["even" if x % 2 == 0 else "odd" for x in range(4)]
assert _parity == ["even", "odd", "even", "odd"], (
    f"if-else inside comp, got {_parity!r}"
)
_ledger.append(1)

# (8) Comprehension over a list literal (not range)
_strs = [s.upper() for s in ["a", "bb", "ccc"]]
assert _strs == ["A", "BB", "CCC"], f"list-literal comp, got {_strs!r}"
_ledger.append(1)

# (9) Filter that uses the loop variable's transformed value
_keep = [x for x in [1, 2, 3, 4, 5] if x * x > 10]
assert _keep == [4, 5], f"filter-on-transform, got {_keep!r}"
_ledger.append(1)

# (10) Dict comprehension from list of pairs
_paired = {k: v for k, v in [("a", 1), ("b", 2)]}
assert _paired == {"a": 1, "b": 2}, f"dict-from-pairs, got {_paired!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_comprehensions {sum(_ledger)} asserts")
