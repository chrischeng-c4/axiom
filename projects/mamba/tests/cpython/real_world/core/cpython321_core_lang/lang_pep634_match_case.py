# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep634_match_case"
# subject = "cpython321.lang_pep634_match_case"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep634_match_case.py"
# status = "filled"
# ///
"""cpython321.lang_pep634_match_case: execute CPython 3.12 seed lang_pep634_match_case"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 634 — structural pattern
# matching (Py 3.10+).
# Surface: literal patterns, OR-patterns (`a | b | c`), guard
# clauses (`case n if ...`), wildcard fallthrough (`_`), sequence
# patterns ([], [x], [x, y]) and rest-binding ([x, *rest]).
def classify(x):
    match x:
        case 0:
            return "zero"
        case 1 | 2 | 3:
            return "small"
        case n if n < 10:
            return "single"
        case _:
            return "big"

def head(seq):
    match seq:
        case []:
            return "empty"
        case [_]:
            return "one"
        case [_, _]:
            return "two"
        case [_, *_]:
            return "many"

_ledger: list[int] = []
# Literal pattern
assert classify(0) == "zero"; _ledger.append(1)
# OR-pattern matches any branch
assert classify(1) == "small"; _ledger.append(1)
assert classify(2) == "small"; _ledger.append(1)
assert classify(3) == "small"; _ledger.append(1)
# Guard clause picks up values not covered by the OR-pattern
assert classify(5) == "single"; _ledger.append(1)
assert classify(7) == "single"; _ledger.append(1)
# Wildcard fallthrough
assert classify(100) == "big"; _ledger.append(1)
# Sequence patterns by length
assert head([]) == "empty"; _ledger.append(1)
assert head([1]) == "one"; _ledger.append(1)
assert head([1, 2]) == "two"; _ledger.append(1)
assert head([1, 2, 3]) == "many"; _ledger.append(1)
assert head([1, 2, 3, 4, 5]) == "many"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep634_match_case {sum(_ledger)} asserts")
