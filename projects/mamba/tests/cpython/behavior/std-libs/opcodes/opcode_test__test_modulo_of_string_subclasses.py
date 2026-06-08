# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_modulo_of_string_subclasses"
# subject = "cpython.test_opcodes.OpcodeTest.test_modulo_of_string_subclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_modulo_of_string_subclasses
"""Auto-ported test: OpcodeTest::test_modulo_of_string_subclasses (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
class MyString(str):

    def __mod__(self, value):
        return 42

assert MyString() % 3 == 42
print("OpcodeTest::test_modulo_of_string_subclasses: ok")
