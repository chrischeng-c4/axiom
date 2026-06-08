# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "behavior"
# case = "misc_test_case__test__all__"
# subject = "cpython.test_plistlib.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_plistlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: MiscTestCase::test__all__ (CPython 3.12 oracle)."""

import plistlib
import unittest
from test import support


not_exported = {"PlistFormat", "PLISTHEADER"}
support.check__all__(unittest.TestCase(), plistlib, not_exported=not_exported)

print("MiscTestCase::test__all__: ok")
