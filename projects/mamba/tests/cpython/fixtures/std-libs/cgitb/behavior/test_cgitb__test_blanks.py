# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "behavior"
# case = "test_cgitb__test_blanks"
# subject = "cpython.test_cgitb.TestCgitb.test_blanks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cgitb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cgitb.py::TestCgitb::test_blanks
"""Auto-ported test: TestCgitb::test_blanks (CPython 3.12 oracle)."""


from test.support.os_helper import temp_dir
from test.support.script_helper import assert_python_failure
from test.support.warnings_helper import import_deprecated
import unittest
import sys


cgitb = import_deprecated('cgitb')


# --- test body ---

assert cgitb.small('') == ''

assert cgitb.strong('') == ''

assert cgitb.grey('') == ''
print("TestCgitb::test_blanks: ok")
