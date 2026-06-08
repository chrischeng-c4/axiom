---
topic: mamba-conformance-lru-cache-cached-property-iter-keyerror
date: '20260424'
project: mamba
branch: mamba
---

## Status

Active session on branch `mamba` (worktree `/Users/chris.cheng/cclab/mamba`). 3 of 4 planned fixes committed at HEAD `486b3ea7`; a fourth fix (KeyError repr/str + `splitlines(keepends=...)` kwarg) is staged but **not yet committed**. Branch is ahead of `origin/mamba` by 75 commits — none pushed.

## Findings

- **`functools.lru_cache` / `cached_property` were identity passthroughs**. Fixed by wiring real caching Instances through the existing decorator + descriptor pipeline. The lru_cache fix also forced a fix to `Var(sym)` lowering for decorated user functions in non-call position (was emitting `FuncRef` which points to the raw JIT code, bypassing the decorator result stored in globals).
- **Forward-referenced class names inside a method body resolved to None** when the referenced class is declared *after* the containing class. Root cause: `user_class_syms` was populated incrementally during the same `for cls in hir.classes` loop that lowered method bodies. Pre-passing the class table fixes it.
- **`raise StopIteration` in user `__next__` leaked past the for-loop exit** because `mb_raise("StopIteration", ...)` sets both the `STOP_ITERATION` flag *and* `CURRENT_EXCEPTION`, but `advance_userdefined_if_applicable` only consumed the flag. Now also clears pending exception.
- **KeyError formatting diverged three ways from CPython**: `print(e)` missed the repr-quoting that `KeyError.__str__` does, `repr(e)` missed `BaseException.__repr__`'s `ClassName(repr(args))` form, and the dict-miss path pre-quoted the message so the new quoting double-up'd. Message is now stored raw; quoting lives in the printer.
- **`s.splitlines(keepends=True)` ignored the kwarg** because the str-method dispatcher pulls `arg(0)` positionally, but the kwargs-only call packs `{"keepends": True}` as a trailing dict. Dispatcher now unwraps that dict for splitlines.
- **Generator `send(non-None)` on a fresh generator is already correct** — probe showed it raises `TypeError: can't send non-None value to a just-started generator`. No change needed. `send_edge_cases.py` conformance also passes unchanged.
- **Pre-existing ARM64 cranelift branch-range SIGABRT** (`compiled_blob.rs:90: diff >> 26 == -1 || diff >> 26 == 0`) and PoisonError cascades still block clean batch runs; isolated per-test runs pass. Not caused by session changes.
- **Handoff CLI emits the file under `~/cclab/main/.score/handoffs/`** even when the current worktree is `mamba`. Manual move is needed (noted in memory `feedback_handoffs_live_on_current_branch.md`).

## Done

Commits on branch `mamba`, all tested and green in isolation via `cargo test -p mamba --test conformance_tests --release -- <name> --test-threads=1`:

- `486b3ea7` — fix(mamba): custom `__iter__` returning a non-self iterator
  - Pre-pass `user_class_syms`/`class_syms` in `crates/mamba/src/lower/hir_to_mir.rs` before any method body lowers.
  - Clear pending exception in `advance_userdefined_if_applicable` in `crates/mamba/src/runtime/iter.rs`.
  - Fixture: `crates/mamba/tests/fixtures/conformance/iterators/custom_iter_non_self.py`.
- `59480dd7` — feat(mamba): functools.cached_property as a real descriptor
  - Added `MethodDecorKind::CachedProperty` + `mb_cached_property_new`/`_get` in `crates/mamba/src/runtime/class.rs`, registered in `symbols.rs`; wired into `is_descriptor` + `invoke_descriptor_get`.
  - Fixture: `crates/mamba/tests/fixtures/conformance/stdlib/functools_cached_property.py`.
- `7b4b6af4` — feat(mamba): functools.lru_cache with real caching + introspection
  - New wrapper + factory + bound-method classes in `crates/mamba/src/runtime/stdlib/functools_mod.rs`.
  - Dispatched via `mb_call0` / `mb_call1_val` / `mb_call_spread` in `class.rs` and `builtins.rs`.
  - `mb_getattr` exposes `cache_info` / `cache_clear`; `mb_call_method` intercepts them.
  - Included fix to `Var(sym)` lowering: decorated user funcs now emit `LoadGlobal` in non-call position (previously `FuncRef` bypassed the decorator's stored wrapper).
  - Fixture: `crates/mamba/tests/fixtures/conformance/stdlib/functools_lru_cache.py`.

**Staged but not committed** — KeyError formatting + `splitlines` keepends:

- `crates/mamba/src/runtime/dict_ops.rs` — new `dict_key_raw_str` helper; `mb_dict_getitem` / pop path use it so the KeyError message is the raw key, not pre-quoted.
- `crates/mamba/src/runtime/string_ops.rs` — `value_to_string` for `KeyError` Instance returns `format!("'{}'", msg)`; `dispatch_str_method` for `splitlines` unwraps a trailing kwargs dict to pull `keepends`.
- `crates/mamba/src/runtime/builtins.rs` — `mb_print` special-cases KeyError to quote the message, and `mb_repr` for exception-subclass Instances falls back to `ClassName(repr(message))`.
- Fixture: `crates/mamba/tests/fixtures/conformance/exceptions/keyerror_repr.py`.

## Next

1. Commit the staged KeyError + splitlines work (message already drafted locally):
   ```
   git add crates/mamba/src/runtime/builtins.rs crates/mamba/src/runtime/dict_ops.rs crates/mamba/src/runtime/string_ops.rs crates/mamba/tests/fixtures/conformance/exceptions/keyerror_repr.py crates/mamba/tests/fixtures/conformance/exceptions/keyerror_repr.expected
   git commit -m "fix(mamba): KeyError str/repr + str.splitlines(keepends=...) kwarg"
   ```
2. Pick up remaining plan work — generator `send(non-None)` probe passed, so task #119 can be closed; focus next rounds on:
   - Async str-concat-return-await SIGSEGV (documented in `32da191f`, needs MIR-level debug).
   - ExceptionGroup / `except*` full PEP 654 gaps (#755).
   - Metaclass `__call__` (xfail at `language/metaclass.py`).
3. Optional cleanup: the pre-existing ARM64 branch-range JIT crash (`compiled_blob.rs:90`) is blocking clean batch runs; not in scope this session.
4. Push to origin when ready:
   ```
   git push origin mamba
   ```

## Criteria

- [ ] `cargo test -p mamba --test conformance_tests --release -- stdlib/functools_lru_cache.py --test-threads=1` passes
- [ ] `cargo test -p mamba --test conformance_tests --release -- stdlib/functools_cached_property.py --test-threads=1` passes
- [ ] `cargo test -p mamba --test conformance_tests --release -- iterators/custom_iter_non_self.py --test-threads=1` passes
- [ ] `cargo test -p mamba --test conformance_tests --release -- exceptions/keyerror_repr.py --test-threads=1` passes (after commit)
- [ ] `cargo test -p mamba --test conformance_tests --release -- stdlib/functools --test-threads=1` — 13/13 pass
- [ ] `cargo build -p mamba --release` succeeds with no new errors
- [ ] no regressions in individually-run decorator/property/iterator/exception fixtures (manual, per pre-existing shared-state note)
