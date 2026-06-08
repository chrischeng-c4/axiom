# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_getattr_tricky"
# subject = "cpython.__init__.ModuleTests.test_module_getattr_tricky"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_getattr_tricky
"""Auto-ported test: ModuleTests::test_module_getattr_tricky (CPython 3.12 oracle)."""


import importlib.machinery
import unittest
import weakref
from test.support import gc_collect
from test.support import import_helper
from test.support.script_helper import assert_python_ok
import sys


ModuleType = type(sys)

class FullLoader:
    pass

class BareLoader:
    pass


# --- test body ---
from test.test_module import bad_getattr3
try:
    bad_getattr3.one
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    bad_getattr3.delgetattr
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
if 'test.test_module.bad_getattr3' in sys.modules:
    del sys.modules['test.test_module.bad_getattr3']
print("ModuleTests::test_module_getattr_tricky: ok")
