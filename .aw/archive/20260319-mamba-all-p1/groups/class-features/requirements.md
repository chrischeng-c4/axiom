---
change: mamba-all-p1
group: class-features
date: 2026-03-19
---

# Requirements

Implement complete Python OOP class features:
- Must-have: `@property` / `@x.setter` / `@x.deleter` (descriptor protocol), `@classmethod` / `@staticmethod`, `__slots__`, common dunder methods (`__repr__`, `__str__`, `__eq__`, `__hash__`), `super()` (both zero-arg and two-arg forms)
- Should-have: `__init_subclass__`, `__class_getitem__` (generic syntax `MyClass[T]`), `__set_name__`, abstract base classes (`abc.ABC`, `@abstractmethod`)
Scope spans: runtime class.rs dunder dispatch, type checker decorator-effect recognition, codegen descriptor protocol call sequence.
Acceptance: standard Python class patterns (dataclass-like, ABC, property-based APIs) work correctly; `super()` resolves method through MRO.
