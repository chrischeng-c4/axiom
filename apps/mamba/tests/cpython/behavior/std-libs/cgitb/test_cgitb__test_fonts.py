# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "behavior"
# case = "test_cgitb__test_fonts"
# subject = "cpython.test_cgitb.TestCgitb.test_fonts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cgitb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cgitb.py::TestCgitb::test_fonts
"""Auto-ported test: TestCgitb::test_fonts (CPython 3.12 oracle)."""


from test.support.os_helper import temp_dir
from test.support.script_helper import assert_python_failure
from test.support.warnings_helper import import_deprecated
import unittest
import sys


cgitb = import_deprecated('cgitb')


# --- test body ---
text = 'Hello Robbie!'

assert cgitb.small(text) == '<small>{}</small>'.format(text)

assert cgitb.strong(text) == '<strong>{}</strong>'.format(text)

assert cgitb.grey(text) == '<font color="#909090">{}</font>'.format(text)
print("TestCgitb::test_fonts: ok")
