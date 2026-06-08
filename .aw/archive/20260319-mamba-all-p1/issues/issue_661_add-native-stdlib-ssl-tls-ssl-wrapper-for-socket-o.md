---
number: 661
title: "Add native stdlib: ssl — TLS/SSL wrapper for socket objects"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-io-networking"
---

# #661 — Add native stdlib: ssl — TLS/SSL wrapper for socket objects

Implement `ssl` module: `SSLContext`, `wrap_socket()`, `create_default_context()` backed by rustls or native-tls.
