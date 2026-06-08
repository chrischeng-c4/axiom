# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep448_dict_spread"
# subject = "cpython321.lang_pep448_dict_spread"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep448_dict_spread.py"
# status = "filled"
# ///
"""cpython321.lang_pep448_dict_spread: execute CPython 3.12 seed lang_pep448_dict_spread"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 448 — dict literal spread
# (**other_dict) merge surface. Only the dict-merge form is asserted;
# the list/tuple/set unpacking forms ([*a, *b], (*a,), {*a, *b}) and
# starred call args (fn(*list, positional)) currently mis-bind on
# mamba and are tracked as separate gaps.
_ledger: list[int] = []
d1 = {"a": 1}
d2 = {"b": 2}
# Spread two dicts in a literal, with an additional explicit pair
merged = {**d1, **d2, "c": 3}
assert merged == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)
assert len(merged) == 3; _ledger.append(1)
# Later-key wins in a left-to-right merge
override = {**{"x": 1}, **{"x": 2}}
assert override == {"x": 2}; _ledger.append(1)
# Empty-dict spread is identity
assert {**{}, "k": "v"} == {"k": "v"}; _ledger.append(1)
assert {**d1} == {"a": 1}; _ledger.append(1)
# Spread plus explicit pair before the spread
mixed = {"a": 0, **d2, "z": 99}
assert mixed == {"a": 0, "b": 2, "z": 99}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep448_dict_spread {sum(_ledger)} asserts")
