//! Shared vendored CPython 3.12 `Lib/` subtree loader (#867).
//!
//! Some stdlib modules are large, stateful, and full of rich-dunder value
//! types that would mean special-casing every dunder in a native Rust
//! dispatcher (see `plistlib_mod`/`fileinput_mod`'s original doc comments).
//! For those, Mamba ships a pure-Python port adapted from real CPython 3.12
//! source and lets Mamba's own compiler execute it, instead of hand-rolling
//! a native shell.
//!
//! Previously each such module carried its own ad-hoc `include_str!` +
//! per-module temp-directory materialization (`fileinput_mod`, `plistlib_mod`,
//! and `long_tail_mod::register_mailbox`), each writing to its own
//! content-hashed directory and each calling `mb_insert_search_path`
//! separately. This module is the single shared mechanism those three were
//! migrated to: every vendored source file is embedded at compile time,
//! materialized ONCE (together) to one shared temp directory at startup, and
//! that one directory is inserted into the import search path so
//! `find_module()` (see `module.rs`) resolves `import X` to the vendored
//! source.
//!
//! ## Precedence
//!
//! `register()` is called once from `stdlib::register_stdlib()`, which runs
//! before `mb_init_search_paths()` reads `PYTHONPATH` (see `main.rs` /
//! `driver/mod.rs`). Combined with `find_module`'s search order, the net
//! precedence for `import X` is:
//!
//! 1. **native** — a module already pre-registered in `MODULES` (e.g. via
//!    `mb_module_register`) is returned straight from `mb_import`'s cache
//!    check; `find_module` (and therefore this tree) is never consulted.
//! 2. **script-dir** — the directory of the currently executing script
//!    (`SCRIPT_DIR`), checked first inside `find_module`.
//! 3. **vendored** — this tree, inserted at `SEARCH_PATHS[0]` (ahead of the
//!    default `"."` entry and anything `mb_init_search_paths` adds later).
//! 4. **user** — the default `"."` entry and any `PYTHONPATH` additions.
//!
//! Adding a module: embed its source below and add a `(name, SRC)` entry to
//! `VENDORED_MODULES`. No other call site changes are needed.

use std::io::Write;
use std::path::PathBuf;

/// `(module_name, embedded source)`. `module_name` becomes `<name>.py` on disk.
const VENDORED_MODULES: &[(&str, &str)] = &[
    ("fileinput", include_str!("py_src/fileinput.py")),
    ("plistlib", include_str!("py_src/plistlib.py")),
    ("mailbox", include_str!("py_src/mailbox.py")),
    // Proves the loader resolves a module Mamba has NO native shell for at
    // all (as opposed to fileinput/plistlib/mailbox, which replaced a native
    // stub) — see #867 AC2.
    ("nturl2path", include_str!("py_src/nturl2path.py")),
    // #868 round 4: colorsys and quopri LANDED after #1007
    // (omitted-trailing-defaults), #1008 (missing-arg TypeError), #1009
    // (string-content/type-name collision), #976 (tuple-unpack swallowed
    // callee exceptions), #977 (repeated-float-call corruption), #953, #945,
    // and #943 all closed. colorsys additionally needed the curated `h: float`
    // compile-time wall in `stdlib_sigs.rs` for `hsv_to_rgb`/`hls_to_rgb` (see
    // there) to keep the two `*_h_as_float_wrong` type fixtures green, since
    // their vendored bodies short-circuit past `h` when `s == 0.0`.
    ("colorsys", include_str!("py_src/colorsys.py")),
    ("quopri", include_str!("py_src/quopri.py")),
    // #868 round 4 bonus attempt (getpass/shlex, both already 0-FAIL native
    // baselines — an architectural de-registration try, not a fail-fix):
    // `shlex` was vendored (its native shell only stubs the `shlex.shlex`
    // class with a None return; split/quote/join are the only real native
    // dispatch) and reverted immediately — the from-source sweep regressed
    // 45 fixtures from 0 FAIL to 12 FAIL (join/quote quoting-style mismatches
    // plus several errors/-dimension raises no longer firing), so the
    // vendored `shlex` class's internal state machine diverges from CPython's
    // in ways the native split/quote/join dispatchers don't. Not re-attempted
    // this round. `getpass` was not attempted at all: its `unix_getpass`
    // vendored body has the exact
    // `try: ... except OSError: fallback = True else: fallback = False`
    // shape already confirmed (getopt bug A above) to corrupt a function-
    // local variable's first-assignment value in mamba, so it was judged
    // near-certain to regress rather than spending a build+sweep cycle to
    // confirm; retry once bug (A) lands.
    // getopt was re-tried this round too (post #976/#1009) and reverted
    // AGAIN — down to 7/31 from-source fails, but every one traces to a
    // fresh, general (not #227-message-wording) core bug, none of them the
    // anticipated blocker:
    //   (A) a local variable whose FIRST assignment is split across a
    //       function-local try/except/else (e.g. `except: x = None` /
    //       `else: x = "foo"`) reads back corrupted (a string body's value
    //       silently becomes `0.0`) — hits `do_longs`'s `optarg` handling
    //       (blocks long_option_inline_value.py).
    //   (B) after a cross-function multi-value tuple-unpack reassignment
    //       (`opts, args = do_shorts(...)`), a subsequent `while args:`
    //       loop-condition re-check doesn't see the new value's truthiness
    //       correctly (an empty-list reassignment still reads truthy, or a
    //       non-empty one reads falsy) — hits getopt()'s own dispatch loop
    //       (blocks short_option_no_arg.py, getopt__args_as__SliceableT_wrong.py,
    //       gnu_getopt__args_as_Sequence_wrong.py, and the Library Reference
    //       walkthrough libref_unix_and_long_options.py, which fails with
    //       `ValueError: not enough values to unpack (expected 2, got 0)` on
    //       the very first `getopt.getopt(...)` call).
    //   (D) mamba doesn't expose class-level `__cause__`/`__context__`/
    //       `__suppress_context__` descriptors on user/module Exception
    //       subclasses at all (confirmed even for a plain unrelated class) —
    //       a pre-existing, general gap the native getopt_mod.rs shell papers
    //       over via an explicit `mb_class_register(..., slots)` call the
    //       vendored plain-Python GetoptError class doesn't get (blocks
    //       getopterror_is_exception_type.py).
    // None of these are filed as narrow single-purpose issues yet in-tree;
    // capture the repros above verbatim for the next retry once cross-module/
    // local-scope try-except-else and while-truthiness-after-unpack bugs are
    // fixed. `py_src/getopt.py` was removed along with the revert (kept
    // nowhere) since it's not referenced.
    // #868 de-registration first batch: `uu` is the sole module from the
    // candidate set (getopt, colorsys, quopri, uu) whose fixtures stayed
    // green from source in that first pass. getopt/colorsys/quopri were
    // tried and reverted then: cross-module dynamic calls into the vendored
    // code corrupted parameter values (p0 #943), and getopt.py/quopri.py's
    // `if __name__ == '__main__':` guards incorrectly fired on plain
    // `import` (separate bug: imported modules got `__name__ == "__main__"`
    // instead of their own module name, p0 #945). uu.py has the same guard
    // but its two live fixtures are compile-time type-checks that never
    // execute the module body, so the latent bug never surfaced there.
    //
    // #868 retry batch (after #943/#945 landed): re-tried getopt/colorsys/
    // quopri and reverted AGAIN — a third, distinct cross-module bug (p0
    // #953: a cross-module call into a module with 2+ top-level function
    // defs corrupts argument values, confirmed for floats) still blocks all
    // three; their fixture dirs regressed to FAIL from source. uu is
    // unaffected in practice because its fixtures never execute the module
    // body at runtime, so it stays vendored. Re-attempt getopt/colorsys/
    // quopri once #953 lands.
    ("uu", include_str!("py_src/uu.py")),
    // #868 retry batch round 2 (after #943/#945/#953 all landed): re-tried
    // getopt/colorsys/quopri from vendored source AGAIN and reverted AGAIN.
    // #953's landed fix only covered a narrow float-argument repro; at least
    // two further distinct cross-module defects remain and still block all
    // three: (1) a tuple-unpack assignment at a call site
    // (`opts, args = inner(...)`) swallows a raised exception from the
    // callee instead of propagating it (blocks getopt's error paths); (2)
    // repeated same-module float-returning calls within a cross-module-
    // entered function corrupt one result to a bit-pattern-as-int artifact,
    // a call shape #953's fix didn't cover (blocks colorsys's hls_to_rgb);
    // (3) a keyword argument forwarded from a parameter default to a
    // bare-name-imported native function (`from binascii import b2a_qp` then
    // `b2a_qp(s, quotetabs=quotetabs, ...)`) gets corrupted in a cross-module
    // call — reproduces even in a single-def module, so unlike (1)/(2) this
    // one is NOT gated by "2+ top-level defs" (blocks quopri). Re-attempt
    // getopt/colorsys/quopri once these land.
    //
    // #868 retry round 3 (after #976/#977/#978 all landed, on top of
    // #943/#945/#953): re-tried getopt/colorsys/quopri from vendored source
    // a third time and reverted AGAIN — each hits a distinct remaining
    // blocker:
    //   - getopt: 7/31 fixtures FAIL. One is a newly-discovered, general,
    //     NOT-part-of-this-family bug: a string literal/value whose *content*
    //     collides with any registered native class/type name (e.g. `"arg"`
    //     collides with the `ast.arg` node type registered by `ast_mod.rs`;
    //     also reproduces for `"keyword"`/`"Module"`/`"Name"`/`"int"`/
    //     `"str"`/`"list"`) mis-dispatches method calls/subscripts on that
    //     value — reproduces standalone with zero imports:
    //     `print('arg'.startswith('-'))` (mamba errors; CPython prints
    //     `False`). getopt's own `posix_stops_at_first_nonoption.py` fixture
    //     trips this via a local `'arg'` string. The other 6 failures are
    //     not yet individually root-caused. Filed as a follow-up (see #868
    //     comments) rather than blocking further debugging here.
    //   - colorsys: DOWN TO A SINGLE remaining failure (was 3/37; the other
    //     two were a compile-time type-wall regression on `hsv_to_rgb`/
    //     `hls_to_rgb`'s `h` param — a curated `stdlib_sigs.rs` fix for that
    //     was prototyped and confirmed to work, but reverted along with
    //     everything else here since it existed only to compensate for the
    //     vendored source path, and colorsys is staying native this round;
    //     revisit alongside the next colorsys retry). The last failure
    //     (`rgb_to_yiq_too_few_args_raises.py`) is OPEN #1008: a cross-module
    //     call missing a required positional argument silently proceeds
    //     instead of raising TypeError.
    //   - quopri: 1/15 non-xfail failure, conclusively OPEN #1007: a
    //     cross-module positional call that omits trailing default kwargs
    //     (`quotetabs`, `header`) forwards garbage instead of the declared
    //     default when `encodestring()` calls native `binascii.b2a_qp`.
    // Re-attempt getopt/colorsys/quopri once #1007/#1008 land (and, for
    // getopt, once the string/type-name collision bug is fixed).
    //
    // #868 round 5: retried getopt after #1014/#1015/#1016 all landed and
    // reverted AGAIN. Both repro'd fixtures (`short_option_with_arg.py`,
    // `libref_unix_and_long_options.py`) now fail via a NEW, much more
    // serious general bug: calling `getopt()`'s own wrapper — a `while
    // args: ... opts, args = do_shorts(opts, ...)` loop whose callee
    // (`do_shorts`) has ITS OWN internal `while optstring != '': ...`
    // loop doing its own tuple-unpack reassignment — corrupts memory
    // NONDETERMINISTICALLY: repeated runs of the identical fixture/repro
    // (no source change) alternate between correct output, wrong output
    // (AssertionError), and outright process crashes (SIGSEGV/SIGBUS/
    // SIGTRAP/"capacity overflow"/"memory allocation ... failed"). Isolated
    // to a same-module nested cross-function while+tuple-unpack call
    // (minimal repro filed; a single-level do_shorts() call alone, or a
    // getopt()-shaped outer loop calling a NON-looping callee, is stable —
    // only the doubly-nested loop-calls-a-looping-callee shape triggers
    // it). This is a deeper sibling of the #1015/#976 family, not covered
    // by their fixes; filed as a new p0 (see #868 comments for the exact
    // repro). Also newly observed (not yet blocking since the crash above
    // already reverts the flip): the two `_SliceableT`/`Sequence`
    // strict-type-wall fixtures need a `stdlib_sigs.rs` entry for
    // `getopt.getopt`/`gnu_getopt`'s `args` param, the same kind of curated
    // wall colorsys's `h: float` needed — same fix shape, just not written
    // yet. getpass was NOT attempted this round: it was gated on the exact
    // same #1014 shape, but since the actual remaining blocker turned out
    // to be this new nested-loop corruption bug instead, getpass's
    // viability is now unknown; do not retry until the new bug lands.
    //
    // #868 round 6: retried getopt after #1018 (nested cross-function
    // while+tuple-unpack memory corruption) landed (jit.rs param_vregs
    // exclusion; verified 50/50 release-mode repro runs clean). getopt now
    // flips clean: the only 2 residual failures were the anticipated
    // `_SliceableT`/`Sequence` type-wall gaps for `getopt`/`gnu_getopt`'s
    // `args` param, fixed with a curated `stdlib_sigs.rs` entry (`Typed`,
    // same shape as colorsys's `h: float` wall) rejecting a bare user
    // instance passed as `args`.
    ("getopt", include_str!("py_src/getopt.py")),
    // #868 round 6: getopt's flip unblocked getpass too (it was gated on
    // the same nested-loop shape via unix_getpass's `contextlib.ExitStack` +
    // `try/except/else` structure). getpass's `termios`/`pwd` imports stay
    // independently native (not de-registered), so the vendored body's
    // `import termios`/`import pwd` resolve to mamba's real shells. All 14
    // behavior-dimension fixtures are XFAIL regardless of backend (mocked
    // CPython unittest tests mamba doesn't support yet); the 4 non-xfail
    // fixtures (3 surface presence checks + 1 compile-time `prompt: str`
    // type wall, independent of registration) stay green from source.
    ("getpass", include_str!("py_src/getpass.py")),
];

/// Materialize the vendored tree (once) and add it to the import search path.
///
/// Deliberately does NOT register a native stub for any of these names: a
/// registered module is pre-seeded into `MODULES` and would win the import
/// cache before `find_module()` is ever consulted, permanently shadowing the
/// real source. With no stub, `import fileinput` (etc.) falls through to the
/// search path and loads the vendored `.py` file. A user-supplied module of
/// the same name in the running script's directory still wins regardless of
/// registration order: `find_module()` consults `SCRIPT_DIR` before
/// `SEARCH_PATHS`.
pub fn register() {
    if let Some(dir) = materialize_vendor_tree() {
        super::super::module::mb_insert_search_path(0, &dir.display().to_string());
    }
}

fn materialize_vendor_tree() -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    // Content-address the directory by hashing every embedded source
    // together, so the tree is (re)written at most once per build and
    // concurrent mamba processes safely share it — the same property the
    // three original per-module materializers each provided individually.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (name, src) in VENDORED_MODULES {
        name.hash(&mut hasher);
        src.hash(&mut hasher);
    }
    let h = hasher.finish();

    let mut dir = std::env::temp_dir();
    dir.push(format!("mamba_vendor_lib_{h:016x}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }

    for (name, src) in VENDORED_MODULES {
        let file = dir.join(format!("{name}.py"));
        // Only (re)write if missing or stale; ignore write races between processes.
        let needs_write = match std::fs::read_to_string(&file) {
            Ok(existing) => existing != *src,
            Err(_) => true,
        };
        if needs_write {
            // Write to a unique temp name then rename, so a partially written
            // file is never observed by a concurrent reader.
            let tmp = dir.join(format!("{name}.{}.tmp", std::process::id()));
            if let Ok(mut f) = std::fs::File::create(&tmp) {
                if f.write_all(src.as_bytes()).is_ok() {
                    let _ = std::fs::rename(&tmp, &file);
                }
            }
        }
    }

    Some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_materialize_vendor_tree_writes_all_modules() {
        let dir = materialize_vendor_tree().expect("vendor tree should materialize");
        for (name, src) in VENDORED_MODULES {
            let path = dir.join(format!("{name}.py"));
            assert!(path.exists(), "{name}.py should exist in the vendored tree");
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                *src,
                "{name}.py on disk should match the embedded source"
            );
        }
    }

    #[test]
    fn test_first_batch_shell_registers_do_not_shadow_vendored_source() {
        crate::runtime::module::cleanup_all_modules();
        register();
        crate::runtime::stdlib::colorsys_mod::register();
        crate::runtime::stdlib::getopt_mod::register();
        crate::runtime::stdlib::getpass_mod::register();
        crate::runtime::stdlib::quopri_mod::register();
        crate::runtime::stdlib::uu_mod::register();

        for name in ["colorsys", "getopt", "getpass", "quopri", "uu"] {
            let registered =
                crate::runtime::module::MODULES.with(|modules| modules.borrow().contains_key(name));
            assert!(
                !registered,
                "{name} must not be pre-seeded in MODULES; a native shell would shadow py_src/{name}.py"
            );
        }

        crate::runtime::module::cleanup_all_modules();
    }
}
