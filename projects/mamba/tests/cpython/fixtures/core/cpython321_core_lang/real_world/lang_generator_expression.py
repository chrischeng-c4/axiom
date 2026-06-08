# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_generator_expression"
# subject = "cpython321.lang_generator_expression"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_generator_expression.py"
# status = "filled"
# ///
"""cpython321.lang_generator_expression: execute CPython 3.12 seed lang_generator_expression"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the generator-expression
# `(expr for x in iterable)` surface. Surface: generator expressions
# materialize via list()/tuple()/set()/dict() into the corresponding
# concrete container; they feed sum/max/min/any/all/len-of-list as a
# lazy iterator producing the expected reduction; they support a
# trailing `if` condition (filter) and multiple `for` clauses (cross
# product, left-clause iterates outer); they can be iterated by a
# `for` loop and the bound values arrive in source order; they can be
# returned from a function and the caller materializes them; they
# compose with `.join(...)` to concatenate stringified results without
# a list intermediate. Companion to lang_comprehensions (list comp)
# and lang_dict_methods (which covers dict literal/comprehension).
_ledger: list[int] = []

# Materialization into list / tuple / set / dict
g = (x * 2 for x in range(5))
assert list(g) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(x for x in range(3)) == [0, 1, 2]; _ledger.append(1)
assert list(x * x for x in range(4)) == [0, 1, 4, 9]; _ledger.append(1)
assert tuple(x for x in [10, 20]) == (10, 20); _ledger.append(1)
assert set(x for x in [1, 2, 2, 3]) == {1, 2, 3}; _ledger.append(1)
assert dict((k, k * 2) for k in [1, 2, 3]) == {1: 2, 2: 4, 3: 6}; _ledger.append(1)

# Reductions over a genexpr — sum / max / min / any / all
assert sum(x for x in range(10)) == 45; _ledger.append(1)
assert sum(x * x for x in range(5)) == 30; _ledger.append(1)
assert max(x for x in [1, 5, 2, 8, 3]) == 8; _ledger.append(1)
assert min(x for x in [5, 1, 8, 2]) == 1; _ledger.append(1)
assert any(x > 3 for x in [1, 2, 4]) == True; _ledger.append(1)
assert all(x > 0 for x in [1, 2, 3]) == True; _ledger.append(1)
assert any(x > 100 for x in [1, 2, 3]) == False; _ledger.append(1)
assert all(x > 5 for x in [1, 2, 3]) == False; _ledger.append(1)

# Trailing-if filter clause
assert list(x for x in range(10) if x % 2 == 0) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(x for x in range(10) if x > 5) == [6, 7, 8, 9]; _ledger.append(1)
assert list(x for x in range(5) if x < 3) == [0, 1, 2]; _ledger.append(1)
assert sum(x for x in range(11) if x % 2 == 0) == 30; _ledger.append(1)

# Multiple-for cross product — outer `for` cycles slower
assert list((a, b) for a in [1, 2] for b in [3, 4]) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)

# Iterated by a `for` loop — values arrive in source order
g = (x for x in [1, 2, 3])
collected = []
for v in g:
    collected.append(v)
assert collected == [1, 2, 3]; _ledger.append(1)

# Returned from a function, materialized by caller
def gen(n: int):
    return (x * x for x in range(n))

assert list(gen(4)) == [0, 1, 4, 9]; _ledger.append(1)
assert sum(gen(5)) == 30; _ledger.append(1)

# Composed with str.join — no list intermediate
assert "-".join(str(x) for x in [1, 2, 3]) == "1-2-3"; _ledger.append(1)
assert ",".join(s.upper() for s in ["a", "b", "c"]) == "A,B,C"; _ledger.append(1)

# Iterating over heterogeneous-looking str source
assert list(s for s in ["a", "b", "c"]) == ["a", "b", "c"]; _ledger.append(1)

# Long-running genexpr materialized
assert len(list(x for x in range(100))) == 100; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_generator_expression {sum(_ledger)} asserts")
