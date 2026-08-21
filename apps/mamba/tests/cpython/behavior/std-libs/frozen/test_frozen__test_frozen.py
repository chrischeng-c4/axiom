# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frozen"
# dimension = "behavior"
# case = "test_frozen__test_frozen"
# subject = "cpython.test_frozen.TestFrozen.test_frozen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frozen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frozen.py::TestFrozen::test_frozen
"""Auto-ported test: TestFrozen::test_frozen (CPython 3.12 oracle)."""


import importlib.machinery
import sys
import unittest
from test.support import captured_stdout, import_helper


'Basic test of the frozen module (source is in Python/frozen.c).'


# --- test body ---
name = '__hello__'
if name in sys.modules:
    del sys.modules[name]
with import_helper.frozen_modules():
    import __hello__
with captured_stdout() as out:
    __hello__.main()

assert out.getvalue() == 'Hello world!\n'
print("TestFrozen::test_frozen: ok")
