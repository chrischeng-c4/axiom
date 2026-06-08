---
change: 1000-patrol
group: add-idlelib-stdlib
date: 2026-04-03
---

# Requirements

Add `idlelib` as a native stdlib module in Mamba. This is the IDLE GUI editor internals package, which is Tkinter-based. Since IDLE is rarely used in production, this should be a minimal stub implementation that provides the module namespace and key submodules so that `import idlelib` succeeds without error. No full Tkinter GUI functionality is expected.
