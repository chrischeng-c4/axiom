# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "finalization_test__test_late_resource_warning"
# subject = "cpython.__init__.FinalizationTest.test_late_resource_warning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_warnings/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::FinalizationTest::test_late_resource_warning
"""Auto-ported test: FinalizationTest::test_late_resource_warning (CPython 3.12 oracle)."""


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
expected = b'sys:1: ResourceWarning: unclosed file '
code = 'f = open(%a)' % __file__
rc, out, err = assert_python_ok('-Wd', '-c', code)

assert err.startswith(expected)
code = 'import warnings; f = open(%a)' % __file__
rc, out, err = assert_python_ok('-Wd', '-c', code)

assert err.startswith(expected)
print("FinalizationTest::test_late_resource_warning: ok")
