---
number: 849
title: "Class features — __slots__, __init_subclass__, properties, classmethods"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "class-features"
---

# #849 — Class features — __slots__, __init_subclass__, properties, classmethods

## Summary

Complete Python class features beyond basic class/method definition:

### Must have
- `@property`, `@x.setter`, `@x.deleter` — descriptor protocol
- `@classmethod`, `@staticmethod` — method types
- `__slots__` — memory-efficient attribute storage
- `__repr__`, `__str__`, `__eq__`, `__hash__` — common dunder methods
- `super()` — both `super().method()` and `super(Class, self).method()`

### Should have
- `__init_subclass__` — subclass hook
- `__class_getitem__` — generic syntax `MyClass[T]`
- `__set_name__` — descriptor naming
- Abstract base classes (`abc.ABC`, `@abstractmethod`)

## Scope

- **Runtime**: class.rs dunder method dispatch
- **Type checker**: Recognize decorator effects on method signatures
- **Codegen**: Descriptor protocol call sequence
