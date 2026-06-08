# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_compare_function_objects"
# subject = "cpython.test_opcodes.OpcodeTest.test_compare_function_objects"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_compare_function_objects
"""Auto-ported test: OpcodeTest::test_compare_function_objects (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
f = eval('lambda: None')
g = eval('lambda: None')

assert f != g
f = eval('lambda a: a')
g = eval('lambda a: a')

assert f != g
f = eval('lambda a=1: a')
g = eval('lambda a=1: a')

assert f != g
f = eval('lambda: 0')
g = eval('lambda: 1')

assert f != g
f = eval('lambda: None')
g = eval('lambda a: None')

assert f != g
f = eval('lambda a: None')
g = eval('lambda b: None')

assert f != g
f = eval('lambda a: None')
g = eval('lambda a=None: None')

assert f != g
f = eval('lambda a=0: None')
g = eval('lambda a=1: None')

assert f != g
print("OpcodeTest::test_compare_function_objects: ok")
