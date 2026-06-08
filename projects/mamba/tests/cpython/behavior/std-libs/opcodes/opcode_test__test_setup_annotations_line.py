# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_setup_annotations_line"
# subject = "cpython.test_opcodes.OpcodeTest.test_setup_annotations_line"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_setup_annotations_line
"""Auto-ported test: OpcodeTest::test_setup_annotations_line (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
try:
    with open(ann_module.__file__, encoding='utf-8') as f:
        txt = f.read()
    co = compile(txt, ann_module.__file__, 'exec')

    assert co.co_firstlineno == 1
except OSError:
    pass
print("OpcodeTest::test_setup_annotations_line: ok")
