# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "finalization_test__test_finalization"
# subject = "cpython.__init__.FinalizationTest.test_finalization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_warnings/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::FinalizationTest::test_finalization
"""Auto-ported test: FinalizationTest::test_finalization (CPython 3.12 oracle)."""


from contextlib import contextmanager
import linecache
import os
import importlib
from io import StringIO
import re
import sys
import textwrap
import types
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.test_warnings.data import package_helper
from test.test_warnings.data import stacklevel as warning_tests
import warnings as original_warnings


py_warnings = import_helper.import_fresh_module('warnings', blocked=['_warnings'])

c_warnings = import_helper.import_fresh_module('warnings', fresh=['_warnings'])

@contextmanager
def warnings_state(module):
    """Use a specific warnings implementation in warning_tests."""
    global __warningregistry__
    for to_clear in (sys, warning_tests):
        try:
            to_clear.__warningregistry__.clear()
        except AttributeError:
            pass
    try:
        __warningregistry__.clear()
    except NameError:
        pass
    original_warnings = warning_tests.warnings
    original_filters = module.filters
    try:
        module.filters = original_filters[:]
        module.simplefilter('once')
        warning_tests.warnings = module
        yield
    finally:
        warning_tests.warnings = original_warnings
        module.filters = original_filters

class BaseTest:
    """Basic bookkeeping required for testing."""

    def setUp(self):
        self.old_unittest_module = unittest.case.warnings
        if '__warningregistry__' in globals():
            del globals()['__warningregistry__']
        if hasattr(warning_tests, '__warningregistry__'):
            del warning_tests.__warningregistry__
        if hasattr(sys, '__warningregistry__'):
            del sys.__warningregistry__
        sys.modules['warnings'] = self.module
        unittest.case.warnings = self.module
        super(BaseTest, self).setUp()

    def tearDown(self):
        sys.modules['warnings'] = original_warnings
        unittest.case.warnings = self.old_unittest_module
        super(BaseTest, self).tearDown()

def setUpModule():
    py_warnings.onceregistry.clear()
    c_warnings.onceregistry.clear()

tearDownModule = setUpModule


# --- test body ---
code = '\nimport warnings\nwarn = warnings.warn\n\nclass A:\n    def __del__(self):\n        warn("test")\n\na=A()\n        '
rc, out, err = assert_python_ok('-c', code)

assert err.decode().rstrip() == '<string>:7: UserWarning: test'
print("FinalizationTest::test_finalization: ok")
