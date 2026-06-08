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
