---
number: 407
title: "mamba: metaclasses and abc (Abstract Base Classes)"
state: closed
labels: [enhancement, P1, crate:mamba]
---

# #407 — mamba: metaclasses and abc (Abstract Base Classes)

## Description

Implement metaclass support and the `abc` module. Many Python frameworks and stdlib modules depend on metaclasses (e.g., `ABCMeta`, `EnumMeta`, dataclasses internals).

## Requirements

### Metaclasses
- R1: `class Foo(metaclass=Meta)` syntax support in class creation
- R2: `__new__` and `__init__` on metaclass called during class definition
- R3: `type()` as default metaclass
- R4: `__init_subclass__` hook

### abc module
- R5: `ABCMeta` metaclass
- R6: `@abstractmethod` decorator — prevents instantiation if not overridden
- R7: `ABC` convenience base class (`class Foo(ABC):`)
- R8: `register()` for virtual subclasses
- R9: `__subclasshook__` for isinstance/issubclass customization

## Priority

P1 — framework foundation; many patterns depend on ABCs.
