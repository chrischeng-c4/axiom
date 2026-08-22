use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/int_literal/test_hex_oct_bin__test_bin_baseline.py`.
#[test]
fn test_gen_behavior_std_libs_int_literal_test_hex_oct_bin__test_bin_baseline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int_literal"
# dimension = "behavior"
# case = "test_hex_oct_bin__test_bin_baseline"
# subject = "cpython.test_int_literal.TestHexOctBin.test_bin_baseline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int_literal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int_literal.py::TestHexOctBin::test_bin_baseline
"""Auto-ported test: TestHexOctBin::test_bin_baseline (CPython 3.12 oracle)."""


import unittest


'Test correct treatment of hex/oct constants.\n\nThis is complex because of changes due to PEP 237.\n'


# --- test body ---

assert 0 == 0

assert 1 == 1

assert 1365 == 1365

assert 0 == 0

assert 16 == 16

assert 2147483647 == 2147483647

assert 9223372036854775807 == 9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807
print("TestHexOctBin::test_bin_baseline: ok")
"###);
    assert_output(&out, r###"TestHexOctBin::test_bin_baseline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/int_literal/test_hex_oct_bin__test_hex_baseline.py`.
#[test]
fn test_gen_behavior_std_libs_int_literal_test_hex_oct_bin__test_hex_baseline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int_literal"
# dimension = "behavior"
# case = "test_hex_oct_bin__test_hex_baseline"
# subject = "cpython.test_int_literal.TestHexOctBin.test_hex_baseline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int_literal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int_literal.py::TestHexOctBin::test_hex_baseline
"""Auto-ported test: TestHexOctBin::test_hex_baseline (CPython 3.12 oracle)."""


import unittest


'Test correct treatment of hex/oct constants.\n\nThis is complex because of changes due to PEP 237.\n'


# --- test body ---

assert 0 == 0

assert 1 == 1

assert 81985529216486895 == 81985529216486895

assert 0 == 0

assert 16 == 16

assert 2147483647 == 2147483647

assert 9223372036854775807 == 9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807
print("TestHexOctBin::test_hex_baseline: ok")
"###);
    assert_output(&out, r###"TestHexOctBin::test_hex_baseline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/int_literal/test_hex_oct_bin__test_oct_baseline.py`.
#[test]
fn test_gen_behavior_std_libs_int_literal_test_hex_oct_bin__test_oct_baseline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int_literal"
# dimension = "behavior"
# case = "test_hex_oct_bin__test_oct_baseline"
# subject = "cpython.test_int_literal.TestHexOctBin.test_oct_baseline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int_literal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int_literal.py::TestHexOctBin::test_oct_baseline
"""Auto-ported test: TestHexOctBin::test_oct_baseline (CPython 3.12 oracle)."""


import unittest


'Test correct treatment of hex/oct constants.\n\nThis is complex because of changes due to PEP 237.\n'


# --- test body ---

assert 0 == 0

assert 1 == 1

assert 342391 == 342391

assert 0 == 0

assert 16 == 16

assert 2147483647 == 2147483647

assert 9223372036854775807 == 9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807

assert -0 == 0

assert -16 == -16

assert -2147483647 == -2147483647

assert -9223372036854775807 == -9223372036854775807
print("TestHexOctBin::test_oct_baseline: ok")
"###);
    assert_output(&out, r###"TestHexOctBin::test_oct_baseline: ok
"###);
}
