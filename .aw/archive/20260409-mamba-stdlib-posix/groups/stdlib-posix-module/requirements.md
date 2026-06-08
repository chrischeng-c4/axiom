---
change: mamba-stdlib-posix
group: stdlib-posix-module
date: 2026-04-09
---

# Requirements

Add native stdlib `posix` module to Mamba runtime.

1. Create `crates/mamba/src/runtime/stdlib/posix_mod.rs` implementing the CPython `posix` module.
2. In CPython, `posix` is the low-level POSIX system call interface that `os` wraps on Unix. The existing `os_mod.rs` already implements most high-level `os` functions. The `posix` module should:
   - Re-export the same callable functions that `os` provides (getpid, getcwd, getenv, listdir, mkdir, remove, rename, makedirs, rmdir, walk, cpu_count)
   - Add `posix.environ` as a live dict of environment variables (populated from `std::env::vars()`)
   - Add `posix.uname_result` stub (returns a tuple of (sysname, nodename, release, version, machine))
   - Add `posix.stat_result` stub
3. Register the module as both `posix` (direct) in `mod.rs`.
4. The `os` module already handles registration as `os` and `os.path`.
5. Follow the `os_mod.rs` dispatch pattern (not the newer extern C ABI from builtins_mod.rs) for consistency with existing os code.
6. Include unit tests for posix.getpid, posix.getcwd, posix.environ, posix.uname_result.
7. Verify no regressions: `cargo test -p mamba --lib -- --test-threads=1 --skip test_long_reference_chain -q`
