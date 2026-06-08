---
number: 658
title: "Add native stdlib: selectors — high-level I/O multiplexing"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-io-networking"
---

# #658 — Add native stdlib: selectors — high-level I/O multiplexing

Implement `selectors` module: `DefaultSelector`, `SelectSelector`, `register()`, `unregister()`, `select()` backed by mio or tokio.
