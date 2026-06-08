# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_raise_class_exceptions"
# subject = "cpython.test_opcodes.OpcodeTest.test_raise_class_exceptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_raise_class_exceptions
"""Auto-ported test: OpcodeTest::test_raise_class_exceptions (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
class AClass(Exception):
    pass

class BClass(AClass):
    pass

class CClass(Exception):
    pass

class DClass(AClass):

    def __init__(self, ignore):
        pass
try:
    raise AClass()
except:
    pass
try:
    raise AClass()
except AClass:
    pass
try:
    raise BClass()
except AClass:
    pass
try:
    raise BClass()
except CClass:

    raise AssertionError('fail')
except:
    pass
a = AClass()
b = BClass()
try:
    raise b
except AClass as v:

    assert v == b
else:

    raise AssertionError('no exception')
try:
    raise DClass(a)
except DClass as v:

    assert isinstance(v, DClass)
else:

    raise AssertionError('no exception')
print("OpcodeTest::test_raise_class_exceptions: ok")
