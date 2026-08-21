# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_filename"
# subject = "cpython.test_codeop.CodeopTests.test_filename"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeop.py::CodeopTests::test_filename
"""Auto-ported test: CodeopTests::test_filename (CPython 3.12 oracle)."""


import unittest
import warnings
from test.support import warnings_helper
from textwrap import dedent
from codeop import compile_command, PyCF_DONT_IMPLY_DEDENT


'\n   Test cases for codeop.py\n   Nick Mathewson\n'


# --- test body ---

assert compile_command('a = 1\n', 'abc').co_filename == compile('a = 1\n', 'abc', 'single').co_filename

assert compile_command('a = 1\n', 'abc').co_filename != compile('a = 1\n', 'def', 'single').co_filename
print("CodeopTests::test_filename: ok")
