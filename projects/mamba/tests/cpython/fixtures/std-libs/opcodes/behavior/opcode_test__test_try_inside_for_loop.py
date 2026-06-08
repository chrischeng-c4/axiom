# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_try_inside_for_loop"
# subject = "cpython.test_opcodes.OpcodeTest.test_try_inside_for_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_try_inside_for_loop
"""Auto-ported test: OpcodeTest::test_try_inside_for_loop (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
n = 0
for i in range(10):
    n = n + i
    try:
        1 / 0
    except NameError:
        pass
    except ZeroDivisionError:
        pass
    except TypeError:
        pass
    try:
        pass
    except:
        pass
    try:
        pass
    finally:
        pass
    n = n + i
if n != 90:

    raise AssertionError('try inside for')
print("OpcodeTest::test_try_inside_for_loop: ok")
