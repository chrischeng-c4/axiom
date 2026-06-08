# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_bad_types"
# subject = "cpython.test_unary.UnaryOpTestCase.test_bad_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_bad_types
"""Auto-ported test: UnaryOpTestCase::test_bad_types (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---
for op in ('+', '-', '~'):

    try:
        eval(op + "b'a'")
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        eval(op + "'a'")
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

try:
    eval('~2j')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    eval('~2.0')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("UnaryOpTestCase::test_bad_types: ok")
