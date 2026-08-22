use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/pow/pow_test__test_bug643260.py`.
#[test]
fn test_gen_behavior_std_libs_pow_pow_test__test_bug643260() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_bug643260"
# subject = "cpython.test_pow.PowTest.test_bug643260"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pow.py::PowTest::test_bug643260
"""Auto-ported test: PowTest::test_bug643260 (CPython 3.12 oracle)."""


import math
import unittest


# --- test body ---
class TestRpow:

    def __rpow__(self, other):
        return None
None ** TestRpow()
print("PowTest::test_bug643260: ok")
"###);
    assert_output(&out, r###"PowTest::test_bug643260: ok
"###);
}
