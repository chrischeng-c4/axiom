---
change: mamba-stdlib-main
group: stdlib-main-module
date: 2026-04-09
---

# Requirements

Add a native stdlib `__main__` module to mamba. In CPython, `__main__` is the module where top-level script code runs and `if __name__ == '__main__'` checks this. Implement as `crates/mamba/src/runtime/stdlib/main_mod.rs` following the `future_mod.rs` pattern. The module must: (1) Register as `__main__` module, (2) Set `__name__` = `"__main__"`, (3) Set `__doc__` = `None`, (4) Set `__loader__` = `None`, (5) Set `__spec__` = `None`.
