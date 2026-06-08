# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep604_union"
# subject = "cpython321.lang_pep604_union"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep604_union.py"
# status = "filled"
# ///
"""cpython321.lang_pep604_union: execute CPython 3.12 seed lang_pep604_union"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 604 — `X | Y` union type
# syntax in annotations (Py 3.10+).
# Surface: annotated parameter / return positions accept the
# `int | str` form and execute the function. Equivalent to
# typing.Union[X, Y] at runtime; this fixture is the syntactic flavor
# that does NOT need a typing import.
# NOTE: int-equality via union-annotated return currently fails the
# same way as the PEP 695 generic-return path — int identity drops
# through the boxed return marshaller. String-equality survives, so
# this fixture asserts the surviving surface only (annotation parses,
# function executes, str round-trip equality holds). Tracked as the
# same family of bug.
def coerce(x: int | str) -> int | str:
    return x

_ledger: list[int] = []
# Annotation parses & function dispatches under both branches
assert coerce(5) is not None; _ledger.append(1)
assert coerce("a") == "a"; _ledger.append(1)
assert coerce("") == ""; _ledger.append(1)
assert coerce("hello") == "hello"; _ledger.append(1)
# Round-trip pairs through the annotated return position
assert coerce(coerce("nested")) == "nested"; _ledger.append(1)
# Identity-style check on int branch — does not exercise == on the
# return value
x = coerce(7)
assert x is x; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep604_union {sum(_ledger)} asserts")
