# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_getattr_errors"
# subject = "cpython.__init__.ModuleTests.test_module_getattr_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_getattr_errors
"""Auto-ported test: ModuleTests::test_module_getattr_errors (CPython 3.12 oracle)."""


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
import test.test_module.bad_getattr as bga
from test.test_module import bad_getattr2

assert bga.x == 1

assert bad_getattr2.x == 1
try:
    bga.nope
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    bad_getattr2.nope
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del sys.modules['test.test_module.bad_getattr']
if 'test.test_module.bad_getattr2' in sys.modules:
    del sys.modules['test.test_module.bad_getattr2']
print("ModuleTests::test_module_getattr_errors: ok")
