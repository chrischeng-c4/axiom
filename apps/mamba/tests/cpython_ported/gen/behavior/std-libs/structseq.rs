use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/structseq/struct_seq_test__test_reference_cycle.py`.
#[test]
fn test_gen_behavior_std_libs_structseq_struct_seq_test__test_reference_cycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_reference_cycle"
# subject = "cpython.test_structseq.StructSeqTest.test_reference_cycle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_reference_cycle
"""Auto-ported test: StructSeqTest::test_reference_cycle (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
script_helper.assert_python_ok('-c', textwrap.dedent('\n            import time\n            t = time.gmtime()\n            type(t).refcyle = t\n        '))
print("StructSeqTest::test_reference_cycle: ok")
"###);
    assert_output(&out, r###"StructSeqTest::test_reference_cycle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/structseq/struct_seq_test__test_repeat.py`.
#[test]
fn test_gen_behavior_std_libs_structseq_struct_seq_test__test_repeat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_repeat"
# subject = "cpython.test_structseq.StructSeqTest.test_repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_repeat
"""Auto-ported test: StructSeqTest::test_repeat (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t1 = time.gmtime()
t2 = 3 * t1
for i in range(len(t1)):

    assert t2[i] == t2[i + len(t1)]

    assert t2[i] == t2[i + 2 * len(t1)]
print("StructSeqTest::test_repeat: ok")
"###);
    assert_output(&out, r###"StructSeqTest::test_repeat: ok
"###);
}
