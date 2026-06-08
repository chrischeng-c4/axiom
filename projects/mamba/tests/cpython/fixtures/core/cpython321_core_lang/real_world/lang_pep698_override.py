# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep698_override"
# subject = "cpython321.lang_pep698_override"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_pep698_override.py"
# status = "filled"
# ///
"""cpython321.lang_pep698_override: execute CPython 3.12 seed lang_pep698_override"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 698 typing.override decorator.
# Behavior: @override is informational (a typing decorator) — at runtime
# it returns the function unchanged. Verify that decorated subclass
# methods still bind correctly and dispatch via inheritance.
from typing import override
_ledger: list[int] = []

class Animal:
    def speak(self) -> str:
        return "..."

class Dog(Animal):
    @override
    def speak(self) -> str:
        return "woof"

class Cat(Animal):
    @override
    def speak(self) -> str:
        return "meow"

assert Animal().speak() == "..."; _ledger.append(1)
assert Dog().speak() == "woof"; _ledger.append(1)
assert Cat().speak() == "meow"; _ledger.append(1)
assert isinstance(Dog(), Animal); _ledger.append(1)
assert isinstance(Cat(), Animal); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep698_override {sum(_ledger)} asserts")
