# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_in_operator"
# subject = "cpython321.lang_in_operator"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_in_operator.py"
# status = "filled"
# ///
"""cpython321.lang_in_operator: execute CPython 3.12 seed lang_in_operator"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `in` / `not in` membership
# operator across containers. Surface: `in` searches list/tuple/set/
# range/dict (key check); substring search on str (and the canonical
# `"" in s == True` edge); `in` honors `==`-equality so `True in [1,
# 2, 3]` (bool/int equivalence in a list-of-int) and tuple-element
# match against a list-of-tuples; works as the if-statement condition
# and as a comprehension filter; `in` composes inside arbitrary
# expressions; `not in` negation is the canonical complement.
# Companion to lang_chained_comparison_bool (which covers
# `a < b < c` chained-compare).
_ledger: list[int] = []

# list / tuple / set membership
assert 2 in [1, 2, 3]; _ledger.append(1)
assert 5 not in [1, 2, 3]; _ledger.append(1)
assert "a" in ["a", "b"]; _ledger.append(1)
assert "z" not in ["a", "b"]; _ledger.append(1)
assert 1 in (1, 2, 3); _ledger.append(1)
assert 4 not in (1, 2, 3); _ledger.append(1)
assert "a" in {"a", "b", "c"}; _ledger.append(1)
assert "z" not in {"a", "b", "c"}; _ledger.append(1)

# str substring search
assert "abc" in "xabcd"; _ledger.append(1)
assert "xyz" not in "xabcd"; _ledger.append(1)
assert "" in "any"; _ledger.append(1)
assert "h" in "hello"; _ledger.append(1)
assert "ell" in "hello"; _ledger.append(1)
assert "world" not in "hello"; _ledger.append(1)

# dict membership = key check
assert "a" in {"a": 1, "b": 2}; _ledger.append(1)
assert "z" not in {"a": 1}; _ledger.append(1)

# range membership
assert 3 in range(10); _ledger.append(1)
assert 10 not in range(10); _ledger.append(1)
assert 0 in range(10); _ledger.append(1)
assert 9 in range(10); _ledger.append(1)
assert -1 not in range(10); _ledger.append(1)

# membership feeds off a list comprehension result
assert 5 in [x for x in range(10)]; _ledger.append(1)
assert 100 not in [x for x in range(10)]; _ledger.append(1)

# bool/int equivalence under ==
assert True in [1, 2, 3]; _ledger.append(1)
assert False in [0, 1, 2]; _ledger.append(1)
assert True not in [2, 3, 4]; _ledger.append(1)

# tuple-element match inside a list of tuples
assert (1, 2) in [(1, 2), (3, 4)]; _ledger.append(1)
assert (5, 6) not in [(1, 2), (3, 4)]; _ledger.append(1)

# membership as the if condition — both branches anchored
hit_cc = False
if 5 in [1, 2, 5]:
    hit_cc = True
assert hit_cc == True; _ledger.append(1)
hit_dd = False
if 99 not in [1, 2, 5]:
    hit_dd = True
assert hit_dd == True; _ledger.append(1)

# membership filter inside a comprehension
assert [x for x in [1, 2, 3, 4] if x in {2, 4}] == [2, 4]; _ledger.append(1)
assert [x for x in range(5) if x not in {0, 2}] == [1, 3, 4]; _ledger.append(1)

# cross-container shape: tuple/set/dict
assert "a" in ("a", "b", "c"); _ledger.append(1)
assert 1 in {1, 2, 3}; _ledger.append(1)
assert "x" in {"x": 1}; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_in_operator {sum(_ledger)} asserts")
