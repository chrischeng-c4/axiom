---
id: mamba-language-core-restore-class-comprehension-cells
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    A[Class body] --> B[Class namespace lowering]
    B --> C[Comprehension function scope]
    B --> D[Method with zero-argument super]
    C --> E[Late-bound comprehension cell]
    D --> F[Synthetic __class__ cell]
    E --> G[Class items and y evaluate]
    F --> H[Method returns enclosing class]
    G --> I[CPython-oracle fixture passes]
    H --> I
```

The fix is a language-core lowering/runtime change: a class-body comprehension owns its loop-variable closure scope, while a method that uses zero-argument `super()` must retain the enclosing class's synthetic `__class__` cell. The two cell paths must coexist without aliasing or consuming one another.
