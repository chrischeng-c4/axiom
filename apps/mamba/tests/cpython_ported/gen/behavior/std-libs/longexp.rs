use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/longexp/long_exp_text__test_longexp.py`.
#[test]
fn test_gen_behavior_std_libs_longexp_long_exp_text__test_longexp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "longexp"
# dimension = "behavior"
# case = "long_exp_text__test_longexp"
# subject = "cpython.test_longexp.LongExpText.test_longexp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_longexp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_longexp.py::LongExpText::test_longexp
"""Auto-ported test: LongExpText::test_longexp (CPython 3.12 oracle)."""


import unittest


# --- test body ---
REPS = 65580
l = eval('[' + '2,' * REPS + ']')

assert len(l) == REPS
print("LongExpText::test_longexp: ok")
"###);
    assert_output(&out, r###"LongExpText::test_longexp: ok
"###);
}
