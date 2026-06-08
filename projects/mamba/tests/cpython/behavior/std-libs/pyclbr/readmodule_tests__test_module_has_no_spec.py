# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "behavior"
# case = "readmodule_tests__test_module_has_no_spec"
# subject = "cpython.test_pyclbr.ReadmoduleTests.test_module_has_no_spec"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyclbr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pyclbr.py::ReadmoduleTests::test_module_has_no_spec
"""Auto-ported test: ReadmoduleTests::test_module_has_no_spec (CPython 3.12 oracle)."""


import sys
from textwrap import dedent
from types import FunctionType, MethodType, BuiltinFunctionType
import pyclbr
from unittest import TestCase, main as unittest_main
from test.test_importlib import util as test_importlib_util
import warnings
from test.support.testcase import ExtraAssertions


'\n   Test cases for pyclbr.py\n   Nick Mathewson\n'

StaticMethodType = type(staticmethod(lambda: None))

ClassMethodType = type(classmethod(lambda c: None))


# --- test body ---
self__modules = pyclbr._modules.copy()
module_name = 'doesnotexist'
assert module_name not in pyclbr._modules
with test_importlib_util.uncache(module_name):
    try:
        pyclbr.readmodule_ex(module_name)
        raise AssertionError('expected ModuleNotFoundError')
    except ModuleNotFoundError:
        pass
print("ReadmoduleTests::test_module_has_no_spec: ok")
