# RUN: parse
# CPython 3.12 test_grammar: class definitions

# Simple class
class Foo:
    pass

# Class with base
class Bar(Foo):
    pass

# Multiple inheritance
class Baz(Foo, Bar):
    pass

# Class with methods
class MyClass:
    x: int = 0

    def __init__(self, x: int):
        self.x = x

    def method(self) -> int:
        return self.x

    @staticmethod
    def static_method():
        pass

    @classmethod
    def class_method(cls):
        pass

    @property
    def value(self):
        return self.x

# Dataclass-style annotations
class Point:
    x: float
    y: float

# Inheritance with super()
class Child(MyClass):
    def __init__(self, x: int, y: int):
        super().__init__(x)
        self.y = y

# Generic class (PEP 695)
class Stack[T]:
    def __init__(self) -> None:
        self._items: list[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)
