# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_lazy_create_annotations"
# subject = "cpython.__init__.ModuleTests.test_lazy_create_annotations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_lazy_create_annotations
"""Auto-ported test: ModuleTests::test_lazy_create_annotations (CPython 3.12 oracle)."""


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
foo = ModuleType('foo')
for i in range(4):

    assert not '__annotations__' in foo.__dict__
    d = foo.__annotations__

    assert '__annotations__' in foo.__dict__

    assert foo.__annotations__ == d

    assert foo.__dict__['__annotations__'] == d
    if i % 2:
        del foo.__annotations__
    else:
        del foo.__dict__['__annotations__']
print("ModuleTests::test_lazy_create_annotations: ok")
