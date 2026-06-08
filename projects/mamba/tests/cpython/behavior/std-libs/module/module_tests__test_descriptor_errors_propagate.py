# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_descriptor_errors_propagate"
# subject = "cpython.__init__.ModuleTests.test_descriptor_errors_propagate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_descriptor_errors_propagate
"""Auto-ported test: ModuleTests::test_descriptor_errors_propagate (CPython 3.12 oracle)."""


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
class Descr:

    def __get__(self, o, t):
        raise RuntimeError

class M(ModuleType):
    melon = Descr()

try:
    getattr(M('mymod'), 'melon')
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("ModuleTests::test_descriptor_errors_propagate: ok")
