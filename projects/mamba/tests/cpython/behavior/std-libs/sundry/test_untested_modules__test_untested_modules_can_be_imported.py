# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sundry"
# dimension = "behavior"
# case = "test_untested_modules__test_untested_modules_can_be_imported"
# subject = "cpython.test_sundry.TestUntestedModules.test_untested_modules_can_be_imported"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sundry.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sundry.py::TestUntestedModules::test_untested_modules_can_be_imported
"""Auto-ported test: TestUntestedModules::test_untested_modules_can_be_imported (CPython 3.12 oracle)."""


import importlib
from test import support
from test.support import import_helper
from test.support import warnings_helper
import unittest


"Do a minimal test of all the modules that aren't otherwise tested."


# --- test body ---
untested = ('encodings',)
with warnings_helper.check_warnings(quiet=True):
    for name in untested:
        try:
            import_helper.import_module('test.test_{}'.format(name))
        except unittest.SkipTest:
            importlib.import_module(name)
        else:

            raise AssertionError('{} has tests even though test_sundry claims otherwise'.format(name))
    import html.entities
    try:
        import tty
    except ImportError:
        if support.verbose:
            print('skipping tty')
print("TestUntestedModules::test_untested_modules_can_be_imported: ok")
