# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_illegal_assignment"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_illegal_assignment"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_illegal_assignment
"""Auto-ported test: DictComprehensionTest::test_illegal_assignment (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
try:
    compile('{x: y for y, x in ((1, 2), (3, 4))} = 5', '<test>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot assign', str(_aR_e))
try:
    compile('{x: y for y, x in ((1, 2), (3, 4))} += 5', '<test>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('illegal expression', str(_aR_e))
print("DictComprehensionTest::test_illegal_assignment: ok")
