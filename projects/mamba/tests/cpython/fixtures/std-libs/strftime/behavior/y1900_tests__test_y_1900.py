# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strftime"
# dimension = "behavior"
# case = "y1900_tests__test_y_1900"
# subject = "cpython.test_strftime.Y1900Tests.test_y_1900"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strftime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strftime.py::Y1900Tests::test_y_1900
"""Auto-ported test: Y1900Tests::test_y_1900 (CPython 3.12 oracle)."""


import calendar
import sys
import re
from test import support
import time
import unittest


'\nUnittest for time.strftime\n'

def fixasctime(s):
    if s[8] == ' ':
        s = s[:8] + '0' + s[9:]
    return s

def escapestr(text, ampm):
    """
    Escape text to deal with possible locale values that have regex
    syntax while allowing regex syntax used for comparison.
    """
    new_text = re.escape(text)
    new_text = new_text.replace(re.escape(ampm), ampm)
    new_text = new_text.replace('\\%', '%')
    new_text = new_text.replace('\\:', ':')
    new_text = new_text.replace('\\?', '?')
    return new_text


# --- test body ---

assert time.strftime('%y', (1900, 1, 1, 0, 0, 0, 0, 0, 0)) == '00'
print("Y1900Tests::test_y_1900: ok")
