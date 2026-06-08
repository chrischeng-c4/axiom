# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "tests__test_unicode_literals"
# subject = "cpython.test.test_future_stmt.test_future_multiple_imports.Tests.test_unicode_literals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_multiple_imports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future_multiple_imports.py::Tests::test_unicode_literals
"""Auto-ported test: Tests::test_unicode_literals (CPython 3.12 oracle)."""


from __future__ import unicode_literals
import unittest


# --- test body ---

assert isinstance('literal', str)
print("Tests::test_unicode_literals: ok")
