---
number: 652
title: "Add native stdlib: atexit — exit handler registration"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-system"
---

# #652 — Add native stdlib: atexit — exit handler registration

Implement `atexit` module with `register()` and `unregister()` backed by Rust process exit hooks.
