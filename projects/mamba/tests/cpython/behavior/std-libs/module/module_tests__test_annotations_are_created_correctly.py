# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_annotations_are_created_correctly"
# subject = "cpython.__init__.ModuleTests.test_annotations_are_created_correctly"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_annotations_are_created_correctly
"""Auto-ported test: ModuleTests::test_annotations_are_created_correctly (CPython 3.12 oracle)."""


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
ann_module4 = import_helper.import_fresh_module('test.typinganndata.ann_module4')

assert '__annotations__' in ann_module4.__dict__
del ann_module4.__annotations__

assert not '__annotations__' in ann_module4.__dict__
print("ModuleTests::test_annotations_are_created_correctly: ok")
