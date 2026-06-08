# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xxlimited"
# dimension = "behavior"
# case = "test_xx_limited__test_xxo_attributes"
# subject = "cpython.test_xxlimited.TestXXLimited.test_xxo_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xxlimited.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xxlimited.py::TestXXLimited::test_xxo_attributes
"""Auto-ported test: TestXXLimited::test_xxo_attributes (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper
import types


xxlimited = import_helper.import_module('xxlimited')

xxlimited_35 = import_helper.import_module('xxlimited_35')


# --- test body ---
module = xxlimited
xxo = module.Xxo()
try:
    xxo.foo
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    del xxo.foo
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
xxo.foo = 1234

assert xxo.foo == 1234
del xxo.foo
try:
    xxo.foo
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("TestXXLimited::test_xxo_attributes: ok")
