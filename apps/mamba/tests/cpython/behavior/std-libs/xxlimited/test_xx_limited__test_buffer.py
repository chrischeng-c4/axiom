# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xxlimited"
# dimension = "behavior"
# case = "test_xx_limited__test_buffer"
# subject = "cpython.test_xxlimited.TestXXLimited.test_buffer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xxlimited.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xxlimited.py::TestXXLimited::test_buffer
"""Auto-ported test: TestXXLimited::test_buffer (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper
import types


xxlimited = import_helper.import_module('xxlimited')

xxlimited_35 = import_helper.import_module('xxlimited_35')


# --- test body ---
module = xxlimited
xxo = module.Xxo()

assert xxo.x_exports == 0
b1 = memoryview(xxo)

assert xxo.x_exports == 1
b2 = memoryview(xxo)

assert xxo.x_exports == 2
b1[0] = 1

assert b1[0] == 1

assert b2[0] == 1
print("TestXXLimited::test_buffer: ok")
