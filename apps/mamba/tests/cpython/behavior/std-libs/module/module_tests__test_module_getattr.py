# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_getattr"
# subject = "cpython.__init__.ModuleTests.test_module_getattr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_getattr
"""Auto-ported test: ModuleTests::test_module_getattr (CPython 3.12 oracle)."""


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
import test.test_module.good_getattr as gga
from test.test_module.good_getattr import test

assert test == 'There is test'

assert gga.x == 1

assert gga.y == 2
try:
    gga.yolo
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Deprecated, use whatever instead', str(_aR_e))

assert gga.whatever == 'There is whatever'
del sys.modules['test.test_module.good_getattr']
print("ModuleTests::test_module_getattr: ok")
