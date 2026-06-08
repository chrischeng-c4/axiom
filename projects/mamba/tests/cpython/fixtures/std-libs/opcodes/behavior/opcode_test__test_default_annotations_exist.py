# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_default_annotations_exist"
# subject = "cpython.test_opcodes.OpcodeTest.test_default_annotations_exist"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_default_annotations_exist
"""Auto-ported test: OpcodeTest::test_default_annotations_exist (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
class C:
    pass

assert C.__annotations__ == {}
print("OpcodeTest::test_default_annotations_exist: ok")
