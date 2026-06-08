---
number: 574
title: "Add complex class definition syntax fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #574 — Add complex class definition syntax fixture

## Context
Class definitions have many syntactic variations.

## Test cases
```python
# Basic
class A: pass
class A(): pass

# Single inheritance
class B(A): pass

# Multiple inheritance
class C(A, B): pass

# Keyword arguments
class D(A, metaclass=type): pass
class E(A, B, metaclass=Meta, **kwargs): pass

# Starred bases (PEP 3115)
bases = [A, B]
# class F(*bases): pass  # This is actually not valid syntax

# Generic class (PEP 695)
class Stack[T]:
    items: list[T]

class Map[K, V](dict[K, V]):
    pass

class Variadic[*Ts]:
    pass

# Complex class body
class Complex:
    # Class variables
    x: int = 0
    y: ClassVar[int] = 0

    # Slots
    __slots__ = ('a', 'b')

    # Init
    def __init__(self): pass

    # Properties
    @property
    def value(self): return self._value

    @value.setter
    def value(self, v): self._value = v

    # Class/static methods
    @classmethod
    def create(cls): pass

    @staticmethod
    def utility(): pass

    # Nested class
    class Inner:
        pass

    # Nested function
    def method(self):
        def helper():
            pass

# Empty class variations
class Empty1: pass
class Empty2: ...
class Empty3:
    pass
class Empty4:
    ...

# Dataclass patterns
from dataclasses import dataclass, field

@dataclass
class Point:
    x: float
    y: float = 0.0
    metadata: dict = field(default_factory=dict)

@dataclass(frozen=True, slots=True)
class FrozenPoint:
    x: float
    y: float
```

## Task
Create `tests/fixtures/parse/edge_cases/class_definitions.py` with `# RUN: parse`.
