# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xxlimited"
# dimension = "behavior"
# case = "test_xx_limited35__test_xxo_demo"
# subject = "cpython.test_xxlimited.TestXXLimited35.test_xxo_demo"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xxlimited.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xxlimited.py::TestXXLimited35::test_xxo_demo
"""Auto-ported test: TestXXLimited35::test_xxo_demo (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper
import types


xxlimited = import_helper.import_module('xxlimited')

xxlimited_35 = import_helper.import_module('xxlimited_35')


# --- test body ---
module = xxlimited_35
xxo = module.Xxo()
other = module.Xxo()

assert xxo.demo('abc') == 'abc'

assert xxo.demo(0) == None
print("TestXXLimited35::test_xxo_demo: ok")
