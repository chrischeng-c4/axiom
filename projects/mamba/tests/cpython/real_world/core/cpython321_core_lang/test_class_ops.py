# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_class_ops"
# subject = "cpython321.test_class_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_class_ops.py"
# status = "filled"
# ///
"""cpython321.test_class_ops: execute CPython 3.12 seed test_class_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for class semantics.
# Surface: __init__, instance attrs, methods returning computed values,
# __repr__ via f-string, single inheritance with method override,
# isinstance against base class.
# Companion to stub/test_class.py — vendored unittest seed.
_ledger: list[int] = []

class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def distance_sq(self) -> int:
        return self.x * self.x + self.y * self.y

    def __repr__(self) -> str:
        return f"Point({self.x}, {self.y})"

p = Point(3, 4)
assert p.x == 3; _ledger.append(1)
assert p.y == 4; _ledger.append(1)
assert p.distance_sq() == 25; _ledger.append(1)
assert repr(p) == "Point(3, 4)"; _ledger.append(1)

class Animal:
    def speak(self) -> str:
        return "..."

class Dog(Animal):
    def speak(self) -> str:
        return "woof"

class Cat(Animal):
    def speak(self) -> str:
        return "meow"

assert Dog().speak() == "woof"; _ledger.append(1)
assert Cat().speak() == "meow"; _ledger.append(1)
assert Animal().speak() == "..."; _ledger.append(1)
assert isinstance(Dog(), Animal); _ledger.append(1)
assert isinstance(Cat(), Animal); _ledger.append(1)
assert not isinstance(Animal(), Dog); _ledger.append(1)

class Counter:
    def __init__(self):
        self.n = 0
    def inc(self) -> None:
        self.n += 1

c = Counter()
c.inc()
c.inc()
c.inc()
assert c.n == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_class_ops {sum(_ledger)} asserts")
