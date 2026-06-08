---
number: 396
title: "feat(mamba): stdlib dataclasses"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #396 — feat(mamba): stdlib dataclasses

## Summary
Implement `dataclasses` standard library module.

## Required
- `@dataclass` decorator — auto-generate `__init__`, `__repr__`, `__eq__`
- `@dataclass(frozen=True)` — immutable instances
- `@dataclass(order=True)` — auto-generate comparison methods
- `field(default=..., default_factory=..., repr=True, compare=True)`
- `fields(instance_or_class)` — introspect fields
- `asdict(instance)`, `astuple(instance)`
- Post-init processing: `__post_init__(self)`
- Inheritance support

## Implementation Notes
- Decorator must transform the class at compile time or runtime
- Needs type annotation introspection to determine fields
- Depends on magic method dispatch (#380) being implemented
