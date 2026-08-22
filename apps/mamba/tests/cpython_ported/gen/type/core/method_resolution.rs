use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/method_resolution/method_self_int_called_with_str.py`.
#[test]
fn test_gen_type_core_method_resolution_method_self_int_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "method_resolution"
# dimension = "type"
# case = "method_self_int_called_with_str"
# subject = "unbound method receiver contract"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: method receiver type mismatch.

CPython 3.12: unbound-method-style call with wrong receiver type
goes through (CPython doesn't enforce `self` annotations).
Mamba: raises TypeError on the receiver-type contract.
"""


class Box:
    def get(self, which: int) -> int:
        return which * 2


try:
    # Call the unbound function with a non-Box self.
    result = Box.get("not_a_box", 3)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
