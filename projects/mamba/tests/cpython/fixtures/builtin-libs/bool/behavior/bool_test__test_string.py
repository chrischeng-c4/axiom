# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_string"
# subject = "cpython.test.test_bool.BoolTest.test_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_string
"""Auto-ported test: BoolTest::test_string (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert 'xyz'.endswith('z') is True

assert 'xyz'.endswith('x') is False

assert 'xyz0123'.isalnum() is True

assert '@#$%'.isalnum() is False

assert 'xyz'.isalpha() is True

assert '@#$%'.isalpha() is False

assert '0123'.isdigit() is True

assert 'xyz'.isdigit() is False

assert 'xyz'.islower() is True

assert 'XYZ'.islower() is False

assert '0123'.isdecimal() is True

assert 'xyz'.isdecimal() is False

assert '0123'.isnumeric() is True

assert 'xyz'.isnumeric() is False

assert ' '.isspace() is True

assert '\xa0'.isspace() is True

assert '\u3000'.isspace() is True

assert 'XYZ'.isspace() is False

assert 'X'.istitle() is True

assert 'x'.istitle() is False

assert 'XYZ'.isupper() is True

assert 'xyz'.isupper() is False

assert 'xyz'.startswith('x') is True

assert 'xyz'.startswith('z') is False
print("BoolTest::test_string: ok")
