---
change: mamba-stdlib-posix
group: stdlib-posix-module
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: Function re-use strategy
- **Answer**: Re-use os_mod.rs dispatch functions. The posix module should delegate to os_mod implementations since CPython os wraps posix. No duplicated logic.

### Q2: environ implementation
- **Answer**: Populate posix.environ as a dict from std::env::vars() at registration time, matching CPython behavior.

### Q3: Dispatch ABI pattern
- **Answer**: Follow the os_mod pattern (from_func dispatch with MbValue args) for consistency, not the newer extern C ABI from builtins_mod.

### Q4: uname implementation
- **Answer**: Use libc::uname on Unix targets with cfg gate. Return a 5-element tuple of strings matching CPython uname_result(sysname, nodename, release, version, machine).

