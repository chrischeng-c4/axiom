# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xxlimited"
# dimension = "behavior"
# case = "test_xx_limited__test_foo"
# subject = "cpython.test_xxlimited.TestXXLimited.test_foo"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xxlimited.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xxlimited.py::TestXXLimited::test_foo
"""Auto-ported test: TestXXLimited::test_foo (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper
import types


xxlimited = import_helper.import_module('xxlimited')

xxlimited_35 = import_helper.import_module('xxlimited_35')


# --- test body ---
module = xxlimited

assert module.foo(1, 2) == 3
print("TestXXLimited::test_foo: ok")
