# /// script
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
