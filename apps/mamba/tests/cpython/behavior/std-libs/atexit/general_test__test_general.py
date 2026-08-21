# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "general_test__test_general"
# subject = "cpython.test_atexit.GeneralTest.test_general"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_atexit.py::GeneralTest::test_general
"""Auto-ported test: GeneralTest::test_general (CPython 3.12 oracle)."""


import atexit
import os
import textwrap
import unittest
from test import support
from test.support import script_helper


# --- test body ---
script = support.findfile('_test_atexit.py')
script_helper.run_test_script(script)
print("GeneralTest::test_general: ok")
