# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_finalization_at_shutdown"
# subject = "cpython.__init__.ModuleTests.test_module_finalization_at_shutdown"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_finalization_at_shutdown
"""Auto-ported test: ModuleTests::test_module_finalization_at_shutdown (CPython 3.12 oracle)."""


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
rc, out, err = assert_python_ok('-c', 'from test.test_module import final_a')

assert not err
lines = out.splitlines()

assert set(lines) == {b'x = a', b'x = b', b'final_a.x = a', b'final_b.x = b', b'len = len', b'shutil.rmtree = rmtree'}
print("ModuleTests::test_module_finalization_at_shutdown: ok")
