# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_multiple_features__test_unicode_literals"
# subject = "cpython.test.test_future_stmt.test_future_multiple_features.TestMultipleFeatures.test_unicode_literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_multiple_features.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_multiple_features.py::TestMultipleFeatures::test_unicode_literals
"""Auto-ported test: TestMultipleFeatures::test_unicode_literals (CPython 3.12 oracle)."""


from __future__ import unicode_literals, print_function
import sys
import unittest
from test import support


# --- test body ---

assert isinstance('', str)
print("TestMultipleFeatures::test_unicode_literals: ok")
