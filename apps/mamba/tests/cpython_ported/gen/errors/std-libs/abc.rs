use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/abc/register_non_class_raises.py`.
#[test]
fn test_gen_errors_std_libs_abc_register_non_class_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "errors"
# case = "register_non_class_raises"
# subject = "abc.ABCMeta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta: register_non_class_raises (errors)."""
import abc

_raised = False
try:
    type('M', (), {}, metaclass=abc.ABCMeta).register(42)
except TypeError:
    _raised = True
assert _raised, "register_non_class_raises: expected TypeError"
print("register_non_class_raises OK")
"###);
    assert_output(&out, r###"register_non_class_raises OK
"###);
}
