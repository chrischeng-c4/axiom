# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dict_merge"
# subject = "cpython321.lang_dict_merge"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_dict_merge.py"
# status = "filled"
# ///
"""cpython321.lang_dict_merge: execute CPython 3.12 seed lang_dict_merge"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the dict-merge surface
# (PEP 584 `|` / `|=` and the `{**a, **b}` spread literal).
# Surface: `d1 | d2` returns a new dict with all of d1's items
# plus all of d2's (RHS wins on overlapping keys); neither operand
# is mutated; merging with `{}` is identity; `d |= other` mutates
# the LHS in place to the merged result; the `{**a, **b}` dict-
# literal spread produces the same composition (RHS wins,
# overlap-with-literal wins by literal-position order), accepts
# `{**{}}` empty-spread, generalizes to three spreads, and copies
# (originals unchanged); `dict()` / `dict(a=1)` /
# `dict([(k,v), ...])` / `dict({k:v})` all construct equivalent
# dicts; `dict.copy()` is a shallow copy (mutating the copy does
# not mutate the original); `dict.update(other)` mutates LHS the
# same way `|=` does.
_ledger: list[int] = []

# `|` operator — disjoint, then overlapping (RHS wins)
d1 = {"a": 1, "b": 2}
d2 = {"c": 3, "d": 4}
assert (d1 | d2) == {"a": 1, "b": 2, "c": 3, "d": 4}; _ledger.append(1)
e1 = {"a": 1, "b": 2}
e2 = {"b": 20, "c": 30}
assert (e1 | e2) == {"a": 1, "b": 20, "c": 30}; _ledger.append(1)

# `|` with empty operand — identity
assert ({} | {"a": 1}) == {"a": 1}; _ledger.append(1)
assert ({"a": 1} | {}) == {"a": 1}; _ledger.append(1)
assert ({} | {}) == {}; _ledger.append(1)

# `|` does not mutate operands
f1 = {"x": 1}
f2 = {"y": 2}
_ = f1 | f2
assert f1 == {"x": 1}; _ledger.append(1)
assert f2 == {"y": 2}; _ledger.append(1)

# `|=` mutates LHS in place
g = {"a": 1}
g |= {"b": 2}
assert g == {"a": 1, "b": 2}; _ledger.append(1)
g2 = {"a": 1}
g2 |= {"a": 99, "b": 2}
assert g2 == {"a": 99, "b": 2}; _ledger.append(1)

# dict() builtin — empty, kwargs, list-of-pairs, dict source
assert dict() == {}; _ledger.append(1)
assert dict(a=1, b=2) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict([("a", 1), ("b", 2)]) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict({"a": 1}) == {"a": 1}; _ledger.append(1)

# Dict-literal spread — disjoint, overlapping (RHS wins by position)
h1 = {"a": 1, "b": 2}
h2 = {"c": 3}
assert {**h1, **h2} == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)
o1 = {"a": 1}
o2 = {"a": 2}
assert {**o1, **o2} == {"a": 2}; _ledger.append(1)

# Spread + literal field
assert {**h1, "d": 4} == {"a": 1, "b": 2, "d": 4}; _ledger.append(1)
# Spread overrides earlier literal of same key
assert {"a": 99, **h1} == {"a": 1, "b": 2}; _ledger.append(1)

# Empty-spread identity
assert {**{}} == {}; _ledger.append(1)
assert {**{}, **h1} == {"a": 1, "b": 2}; _ledger.append(1)

# Three-way spread
i1 = {"a": 1}
i2 = {"b": 2}
i3 = {"c": 3}
assert {**i1, **i2, **i3} == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)
# Spread copies — original i1 unchanged
assert i1 == {"a": 1}; _ledger.append(1)

# dict.copy — shallow, mutating copy doesn't touch original
j = {"a": 1, "b": 2}
k = j.copy()
assert k == {"a": 1, "b": 2}; _ledger.append(1)
k["a"] = 99
assert j["a"] == 1; _ledger.append(1)
assert k["a"] == 99; _ledger.append(1)

# dict.update — mutates LHS like `|=`
m = {"a": 1}
m.update({"b": 2})
assert m == {"a": 1, "b": 2}; _ledger.append(1)
m.update({"a": 99, "c": 3})
assert m == {"a": 99, "b": 2, "c": 3}; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dict_merge {sum(_ledger)} asserts")
