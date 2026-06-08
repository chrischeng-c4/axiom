# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep584_dict_union"
# subject = "cpython321.lang_pep584_dict_union"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep584_dict_union.py"
# status = "filled"
# ///
"""cpython321.lang_pep584_dict_union: execute CPython 3.12 seed lang_pep584_dict_union"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 584 dict union operators
# (`|` and `|=`) introduced in CPython 3.9.
# Surface: the binary `|` operator merges two dicts producing a new
# dict; on key collisions the right-hand operand wins (so the order
# of operands matters). The in-place `|=` mutates the left operand
# with the same right-precedence semantics.
_ledger: list[int] = []
d1 = {"a": 1, "b": 2}
d2 = {"b": 3, "c": 4}
# Right-hand operand wins on collisions
merged = d1 | d2
assert merged == {"a": 1, "b": 3, "c": 4}; _ledger.append(1)
# Reversing operands flips which value of "b" survives
flipped = d2 | d1
assert flipped == {"b": 2, "c": 4, "a": 1}; _ledger.append(1)
# `|` is non-mutating — original dicts are unchanged
assert d1 == {"a": 1, "b": 2}; _ledger.append(1)
assert d2 == {"b": 3, "c": 4}; _ledger.append(1)
# In-place update with `|=`
d3 = {"x": 1}
d3 |= {"y": 2}
assert d3 == {"x": 1, "y": 2}; _ledger.append(1)
# `|=` with a collision: right wins
d4 = {"k": 1, "v": 2}
d4 |= {"k": 99}
assert d4 == {"k": 99, "v": 2}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep584_dict_union {sum(_ledger)} asserts")
