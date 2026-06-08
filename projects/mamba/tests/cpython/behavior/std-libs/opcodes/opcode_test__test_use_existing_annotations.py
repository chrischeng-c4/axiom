# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_use_existing_annotations"
# subject = "cpython.test_opcodes.OpcodeTest.test_use_existing_annotations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_use_existing_annotations
"""Auto-ported test: OpcodeTest::test_use_existing_annotations (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
ns = {'__annotations__': {1: 2}}
exec('x: int', ns)

assert ns['__annotations__'] == {'x': int, 1: 2}
print("OpcodeTest::test_use_existing_annotations: ok")
