//! ① Type-wall PoC: a tiny hardcoded table of stdlib call signatures so that a
//! wrong-typed *stdlib* argument is rejected at compile time, reusing the same
//! value-vs-annotation rejection loop that already rejects `x: int = "3"`.
//!
//! This is a proof-of-concept *path*, not a complete typeshed import. The real
//! version would be generated from `vendor/typeshed`. The closed [`CoreTy`] enum
//! is deliberately scalar-only: anything we cannot represent as a concrete
//! scalar (protocols, unions, typevars, overloads, buffers) collapses to
//! [`CoreTy::Unknown`], which the hook *skips* — guaranteeing zero false
//! positives on correct calls.

/// Closed set of argument types the PoC table can express. Anything richer
/// (Optional, Union, Protocol, TypeVar, overload, ReadableBuffer, SupportsIndex)
/// must be encoded as [`CoreTy::Unknown`] so the hook never rejects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreTy {
    Int,
    Float,
    Str,
    Bytes,
    Bool,
    None,
    /// A NOMINAL/protocol type contract (a named typeshed type that is neither a
    /// concrete scalar nor `object`/`Any`/`Incomplete`): `os.PathLike`,
    /// `_SupportsFloatOrIndex`, `BaseException`, etc. The hook does not treat it
    /// as a scalar, but it rejects a *bare* user class instance (no bases, no
    /// methods — `class _W: pass`) passed here, since such a value can satisfy
    /// neither a protocol (no dunders) nor a nominal type (no superclass). Any
    /// class with a base or a method, and every non-class value, is skipped — so
    /// it stays false-positive-clean.
    Typed,
    /// Not a concrete scalar — never enforce against this. Catch-all for every
    /// non-scalar typeshed annotation (unions, subscripts, typevars, buffers,
    /// object/Any, etc.).
    Unknown,
}

/// What kind of callee a signature describes, used to disambiguate the lookup
/// against import provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigKind {
    /// Module-level free function: `os.strerror`, `base64.b64encode`.
    ModuleFn,
    /// Instance method: `HTMLParser.handle_entityref` (qualifier = class name).
    Method,
}

/// A single positional parameter's enforceable contract.
#[derive(Debug, Clone, Copy)]
pub struct ParamSig {
    pub name: &'static str,
    pub ty: CoreTy,
    /// `*args` / `*` boundary: the hook stops enforcement at the first star
    /// param and never enforces past it.
    pub star: bool,
}

/// One stdlib callable signature.
#[derive(Debug, Clone, Copy)]
pub struct StdlibSig {
    /// Dotted module path, e.g. `"os"`, `"html.parser"`,
    /// `"multiprocessing.reduction"`.
    pub module: &'static str,
    /// For [`SigKind::Method`], the owning class name (e.g. `"HTMLParser"`).
    /// Empty for module functions.
    pub qualifier: &'static str,
    /// The callable's own name.
    pub name: &'static str,
    pub kind: SigKind,
    pub params: &'static [ParamSig],
    /// Whether this signature has *any* enforceable (concrete-scalar) param. If
    /// false, the hook skips it wholesale (kept in the table as a documented
    /// negative test that it is NOT rejected).
    pub enforceable: bool,
}

const fn p(name: &'static str, ty: CoreTy) -> ParamSig {
    ParamSig { name, ty, star: false }
}

/// The PoC signature table. Hardcoded; the production version regenerates this
/// from typeshed.
pub const STDLIB_SIGS: &[StdlibSig] = &[
    // POSITIVE: os.strerror(code: int) — bare-scalar module fn, enforceable.
    StdlibSig {
        module: "os",
        qualifier: "",
        name: "strerror",
        kind: SigKind::ModuleFn,
        params: &[p("code", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: os.getenv(key: str, default=...) — bare-scalar module fn.
    // Only `key` is concrete (str); `default` is Unknown, so the hook stops
    // enforcing after the first non-scalar param.
    StdlibSig {
        module: "os",
        qualifier: "",
        name: "getenv",
        kind: SigKind::ModuleFn,
        params: &[p("key", CoreTy::Str), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: multiprocessing.reduction.duplicate(handle: int, ...).
    StdlibSig {
        module: "multiprocessing.reduction",
        qualifier: "",
        name: "duplicate",
        kind: SigKind::ModuleFn,
        params: &[p("handle", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: html.parser.HTMLParser.handle_entityref(name: str) — method.
    StdlibSig {
        module: "html.parser",
        qualifier: "HTMLParser",
        name: "handle_entityref",
        kind: SigKind::Method,
        params: &[p("name", CoreTy::Str)],
        enforceable: true,
    },
    // NEGATIVE: base64.b64encode(s: ReadableBuffer, altchars=...) — `s` is a
    // buffer protocol -> Unknown, so this is NOT enforceable. Kept as a
    // regression guard that `b64encode(123)` is never rejected.
    StdlibSig {
        module: "base64",
        qualifier: "",
        name: "b64encode",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Unknown), p("altchars", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: math.factorial(x: SupportsIndex) — protocol -> Unknown, NOT
    // enforceable. Kept as a regression guard that `factorial(obj)` and
    // `factorial(3.0)` are never rejected by this table.
    StdlibSig {
        module: "math",
        qualifier: "",
        name: "factorial",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: calendar.setfirstweekday(firstweekday) — CPython's body is
    // `if not MONDAY <= firstweekday <= SUNDAY`, so a str argument is a
    // RUNTIME TypeError (from the int/str comparison), not a compile-time
    // reject. The runtime dispatcher raises it; keep the wall out of the way.
    StdlibSig {
        module: "calendar",
        qualifier: "",
        name: "setfirstweekday",
        kind: SigKind::ModuleFn,
        params: &[p("firstweekday", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: fnmatch.translate(pat) — `translate(123)` is a RUNTIME
    // TypeError (normcase raises it); the dispatcher models that contract.
    StdlibSig {
        module: "fnmatch",
        qualifier: "",
        name: "translate",
        kind: SigKind::ModuleFn,
        params: &[p("pat", CoreTy::Unknown)],
        enforceable: false,
    },
];

/// Look up a signature by `(module, qualifier, name)`. `qualifier` is `""` for
/// module functions and the class name for methods.
///
/// The curated [`STDLIB_SIGS`] table takes precedence (it is an explicit,
/// human-verified override — including documented negative guards). On a miss
/// we fall back to the typeshed-derived [`STDLIB_SIGS_GENERATED`] table, whose
/// rows are conservatively `enforceable=false` for anything non-scalar /
/// overloaded / variadic. Either way the call-site hook only ever enforces
/// when `enforceable=true` AND the actual argument is a concrete scalar, so a
/// fallback row never introduces a false positive on a correct call.
pub fn get(module: &str, qualifier: &str, name: &str) -> Option<&'static StdlibSig> {
    STDLIB_SIGS
        .iter()
        .find(|s| s.module == module && s.qualifier == qualifier && s.name == name)
        .or_else(|| {
            super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED
                .iter()
                .find(|s| s.module == module && s.qualifier == qualifier && s.name == name)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_module_fn() {
        let s = get("os", "", "strerror").expect("strerror present");
        assert!(s.enforceable);
        assert_eq!(s.params[0].ty, CoreTy::Int);
        assert_eq!(s.kind, SigKind::ModuleFn);
    }

    #[test]
    fn lookup_method() {
        let s = get("html.parser", "HTMLParser", "handle_entityref")
            .expect("method present");
        assert_eq!(s.kind, SigKind::Method);
        assert_eq!(s.params[0].ty, CoreTy::Str);
    }

    #[test]
    fn negative_not_enforceable() {
        assert!(!get("base64", "", "b64encode").unwrap().enforceable);
        assert!(!get("math", "", "factorial").unwrap().enforceable);
    }

    #[test]
    fn qualifier_disambiguates() {
        // Method lookup with empty qualifier must miss.
        assert!(get("html.parser", "", "handle_entityref").is_none());
        // Module-fn lookup with a qualifier must miss.
        assert!(get("os", "HTMLParser", "strerror").is_none());
    }

    #[test]
    fn unknown_misses() {
        assert!(get("os", "", "nonexistent").is_none());
        assert!(get("nope", "", "strerror").is_none());
    }

    // --- Generated typeshed table -----------------------------------------

    #[test]
    fn generated_table_is_nonempty_and_consulted() {
        use super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        assert!(
            STDLIB_SIGS_GENERATED.len() > 1000,
            "generated table should hold thousands of typeshed sigs, got {}",
            STDLIB_SIGS_GENERATED.len(),
        );
        // At least some rows must be enforceable scalars; most are Unknown-skipped.
        let enf = STDLIB_SIGS_GENERATED.iter().filter(|s| s.enforceable).count();
        assert!(enf > 100, "expected hundreds of enforceable scalar sigs, got {enf}");
    }

    #[test]
    fn generated_enforceable_rows_have_a_scalar_and_no_star() {
        // The invariant: every row the hook will ENFORCE must (a) carry at least
        // one concrete-scalar (Int/Float/Str) param to check, and (b) have no
        // star param (positional alignment past `*args` is uncertain). A row MAY
        // also carry Unknown/None/Bytes params — `core_ty_to_type_id` maps those
        // to None, so the hook SKIPS them (advancing the positional index) and
        // never rejects against them. This is what lets a scalar param sitting
        // BEHIND an Unknown param enforce at its real position; a full uncapped
        // 28k-fixture ② FP scan confirms 0 false positives from the skipped
        // params. (Earlier the generator truncated enforceable rows to their
        // leading scalar prefix to satisfy a stricter all-scalar invariant.)
        use super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        for s in STDLIB_SIGS_GENERATED.iter().filter(|s| s.enforceable) {
            assert!(!s.params.is_empty(), "{}.{} enforceable but no params", s.module, s.name);
            assert!(
                s.params.iter().any(|p| matches!(
                    p.ty, CoreTy::Int | CoreTy::Float | CoreTy::Str | CoreTy::Typed
                )),
                "{}.{} enforceable with no checkable (scalar/Typed) param", s.module, s.name,
            );
            for prm in s.params {
                assert!(!prm.star, "{}.{} enforceable with a star param", s.module, s.name);
            }
        }
    }

    #[test]
    fn curated_overrides_generated() {
        // The 6 curated rows win over any generated row of the same key, and the
        // generated table is reachable on a curated miss.
        let s = get("os", "", "strerror").unwrap();
        assert!(s.enforceable, "curated os.strerror override must stay enforceable");
        // A purely generated lookup (not in the curated 6) must resolve.
        assert!(
            super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED
                .iter()
                .any(|s| s.module == "os" && s.qualifier.is_empty()),
            "generated table should contain os module fns",
        );
    }

    /// Regenerable contract (fixture_lint-style): the checked-in
    /// `stdlib_sigs_generated.rs` must be byte-for-byte reproducible by
    /// `type_wall_gen.py --emit-rust`. Skips gracefully if `python3.12` or the
    /// vendored typeshed is unavailable (CI without the harness toolchain).
    #[test]
    fn generated_table_is_regenerable() {
        use std::path::Path;
        use std::process::Command;
        let manifest = env!("CARGO_MANIFEST_DIR");
        let gen = Path::new(manifest)
            .join("tests/harness/cpython/tools/type_wall_gen.py");
        let typeshed = Path::new(manifest).join("vendor/typeshed/stdlib");
        if !gen.exists() || !typeshed.exists() {
            eprintln!("skip: harness generator / typeshed not present");
            return;
        }
        let out = match Command::new("python3.12").arg(&gen).arg("--check-rust").output() {
            Ok(o) => o,
            Err(_) => {
                eprintln!("skip: python3.12 not available");
                return;
            }
        };
        assert!(
            out.status.success(),
            "stdlib_sigs_generated.rs is stale — re-run \
             `python3.12 tests/harness/cpython/tools/type_wall_gen.py --emit-rust`.\n\
             stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}
