//! Py3.12 conformance + regression tests for variadic call lowering — issue #2098.
//!
//! #2098 originally surfaced from Phase 2 Task #6 struct conformance:
//! `self.assertRaises(struct.error, struct.calcsize, 'Z')` emitted a
//! Cranelift call with 5 args against a `(i64) -> i64`-declared thunk,
//! aborting `define_function` with a verifier error. The fix landed via
//! #1696's call-site arity reshape (`emit_internal_call` /
//! `emit_extern_call` truncate-over / pad-under). A pure-JIT compile
//! regression test pins the verifier-clean property at
//! `tests/jit_tests.rs::test_jit_issue_2098_variadic_assert_raises_no_verifier_abort`.
//!
//! This file adds the end-to-end semantic regression: not just that
//! the JIT module compiles, but that variadic dispatch (`struct.pack`
//! with splatted args, `assertRaises` with forwarded args) produces
//! semantically correct runtime behaviour via the standard
//! `jit_capture` pipeline used by all conformance tests.
//!
//! @issue #2098

use super::{assert_output, jit_capture};

#[test]
fn test_struct_pack_unpack_splat_roundtrip() {
    // Variadic splat: struct.pack(fmt, *args) must deliver all unpacked
    // args to the native shim. Original #2098 symptom was `sum(0..63)`
    // collapsing to 0 — args silently truncated by call-site reshape.
    let out = jit_capture(
        r#"import struct
values = tuple(range(8))
packed = struct.pack("8B", *values)
unpacked = struct.unpack("8B", packed)
print(unpacked == values)
print(sum(unpacked))
"#,
    );
    assert_output(&out, "True\n28\n");
}

#[test]
fn test_struct_calcsize_assert_raises_variadic_shape() {
    // The exact 3-arg form from the #2098 symptom:
    // `assertRaises(exc_type, callable, arg)`. Originally tripped the
    // Cranelift verifier; must now compile and execute cleanly.
    let out = jit_capture(
        r#"import unittest
import struct

class T(unittest.TestCase):
    def test_calcsize_z_format(self):
        self.assertRaises(struct.error, struct.calcsize, "Z")

t = T("test_calcsize_z_format")
t.test_calcsize_z_format()
print("ok")
"#,
    );
    assert_output(&out, "ok\n");
}

#[test]
fn test_struct_pack_variadic_large_arity() {
    // Stress: 16-arg variadic splat. Larger than typical 3-4 arg
    // assertRaises shapes — catches arity-handling regressions where
    // a small fixed limit might paper over the smaller case.
    let out = jit_capture(
        r#"import struct
values = tuple(range(16))
packed = struct.pack("16B", *values)
unpacked = struct.unpack("16B", packed)
print(unpacked == values)
"#,
    );
    assert_output(&out, "True\n");
}
