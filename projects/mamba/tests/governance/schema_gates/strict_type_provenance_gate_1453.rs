// HANDWRITE-BEGIN gap="missing-generator:mamba-strict-type-provenance-governance-gate" tracker="#1453" reason="The raw callable and producer contract inventory is a repository-level semantic gate."
//! Fail-closed source inventory for the strict raw-or-boxed Int proof (#1453).

use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn read(relative: &str) -> String {
    fs::read_to_string(project_root().join(relative))
        .unwrap_or_else(|error| panic!("read provenance inventory source {relative}: {error}"))
}

fn raw_address_invocations(source: &str) -> Vec<(usize, &str)> {
    source
        .match_indices("std::mem::transmute(")
        .map(|(offset, _)| {
            let end = source[offset..]
                .find(')')
                .map(|length| offset + length + 1)
                .unwrap_or(source.len());
            (offset, &source[offset..end])
        })
        .collect()
}

fn classified_raw_address_call(file: &str, source: &str, offset: usize, call: &str) -> bool {
    // The central dynamic dispatcher owns the raw JIT return ABI.  Its
    // individual arity arms intentionally contain transmute(raw_addr).
    if call.contains("raw_addr") && file.ends_with("runtime/builtins/mod.rs") {
        return true;
    }

    // Coroutine bodies use their separately declared `(i64) -> i64` ABI and
    // do not transport a dynamic return-owner token.  Keep the exemption
    // named and narrow rather than treating arbitrary async transmute as safe.
    if file.ends_with("runtime/async_rt.rs") {
        let prefix = &source[..offset];
        return prefix.rfind("fn decode_coroutine_body").is_some();
    }

    // The class test fixture invokes a registered native __set_name__ probe;
    // it is not a JIT Python-return ABI and remains a named test-only exemption.
    if file.ends_with("runtime/class/mod.rs")
        && call.contains("addr as usize")
        && source[offset.saturating_sub(900)..offset].contains("CALLABLE_REGISTRY")
    {
        return true;
    }

    // All remaining dynamic `addr` casts in the builtins/class paths must be
    // a native flat-args or variadic ABI, never a JIT Python return route.
    let context_start = offset.saturating_sub(900);
    let context = &source[context_start..offset];
    context.contains("is_native_func") || context.contains("is_variadic_func")
}

fn assert_all_raw_address_calls_are_classified(file: &str, source: &str) {
    let unknown: Vec<_> = raw_address_invocations(source)
        .into_iter()
        .filter_map(|(offset, call)| {
            (!classified_raw_address_call(file, source, offset, call))
                .then_some((offset, call.to_string()))
        })
        .collect();
    assert!(
        unknown.is_empty(),
        "unclassified raw callable invocation(s) in {file}: {unknown:#?}; route JIT returns through dispatch_jit_frame/dispatch_jit_method_return or record a named native ABI exemption"
    );
}

#[test]
fn raw_or_boxed_provenance_inventory_is_complete() {
    let paths = [
        "src/runtime/builtins/mod.rs",
        "src/runtime/class/mod.rs",
        "src/runtime/async_rt.rs",
    ];
    for path in paths {
        let source = read(path);
        assert_all_raw_address_calls_are_classified(path, &source);
    }

    // The producer owner contract is the canonical registration point.  Both
    // Cranelift backends must keep explicit lowering arms for every producer
    // that can carry a raw-or-boxed Int physical value.
    let owners = read("src/mir/producer_owner.rs");
    let jit = read("src/codegen/cranelift/jit.rs");
    let object = read("src/codegen/cranelift/mod.rs");
    for producer in [
        "LoadConst",
        "LoadGlobal",
        "LoadCell",
        "LoadCapture",
        "GetAttr",
        "GetItem",
        "BinOp",
        "UnaryOp",
        "CheckedAdd",
        "CheckedSub",
        "CheckedMul",
        "Copy",
        "Call",
        "CallExtern",
    ] {
        assert!(owners.contains(producer), "owner contract omits {producer}");
        assert!(
            jit.contains(&format!("MirInst::{producer}")),
            "JIT omits {producer}"
        );
        assert!(
            object.contains(&format!("MirInst::{producer}")),
            "Object backend omits {producer}"
        );
    }
    for extern_contract in [
        "mb_pow_int",
        "mb_unbox_int_if_boxed",
        "mb_unbox_inline_int_if_boxed",
    ] {
        assert!(
            owners.contains(extern_contract),
            "missing explicit extern provenance contract: {extern_contract}"
        );
    }

    let bypass = "fn bypass(addr: usize) { let call: extern \"C\" fn() -> i64 = unsafe { std::mem::transmute(addr) }; let _ = call(); }";
    let (offset, call) = raw_address_invocations(bypass)
        .into_iter()
        .next()
        .expect("synthetic bypass has raw invocation");
    assert!(
        !classified_raw_address_call("synthetic.rs", bypass, offset, call),
        "synthetic unclassified JIT bypass must fail closed"
    );
}

// marker: missing-generator:mamba-strict-type-provenance-governance-gate
// HANDWRITE-END
