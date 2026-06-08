# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep673_self_type"
# subject = "cpython321.lang_pep673_self_type"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep673_self_type.py"
# status = "filled"
# ///
"""cpython321.lang_pep673_self_type: execute CPython 3.12 seed lang_pep673_self_type"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 673 — `Self` type
# (CPython 3.11+).
# Surface: a method annotated to return `Self` returns the current
# instance, supporting fluent-builder chains. Both the string-form
# `"Self"` (forward reference) and the bare `Self` (after `from
# typing import Self`) annotation positions are exercised.
from typing import Self
_ledger: list[int] = []

class Builder:
    def __init__(self):
        self.parts: list[str] = []
    def add(self, x: str) -> Self:
        self.parts.append(x)
        return self
    def build(self) -> str:
        return "-".join(self.parts)

# Fluent chain: each .add returns self, so build() sees all parts
b = Builder()
result = b.add("a").add("b").add("c").build()
assert result == "a-b-c"; _ledger.append(1)
# All four method returns reference the same underlying instance
b2 = Builder()
r1 = b2.add("x")
r2 = r1.add("y")
assert r1 is b2; _ledger.append(1)
assert r2 is b2; _ledger.append(1)
# Final state is the accumulated parts list
assert b2.parts == ["x", "y"]; _ledger.append(1)
# A chain starting from a fresh instance does not bleed from a prior one
b3 = Builder()
assert b3.build() == ""; _ledger.append(1)
# Chaining survives across multiple calls in a single expression
b4 = Builder()
assert b4.add("1").add("2").build() == "1-2"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep673_self_type {sum(_ledger)} asserts")
