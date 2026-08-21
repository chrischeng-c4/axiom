# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "behavior"
# case = "test_module__test_constants"
# subject = "cpython.test_termios.TestModule.test_constants"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_termios.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_termios.py::TestModule::test_constants
"""Auto-ported test: TestModule::test_constants (CPython 3.12 oracle)."""


import errno
import os
import sys
import tempfile
import unittest
from test.support.import_helper import import_module


termios = import_module('termios')


# --- test body ---

assert isinstance(termios.B0, int)

assert isinstance(termios.B38400, int)

assert isinstance(termios.TCSANOW, int)

assert isinstance(termios.TCSADRAIN, int)

assert isinstance(termios.TCSAFLUSH, int)

assert isinstance(termios.TCIFLUSH, int)

assert isinstance(termios.TCOFLUSH, int)

assert isinstance(termios.TCIOFLUSH, int)

assert isinstance(termios.TCOOFF, int)

assert isinstance(termios.TCOON, int)

assert isinstance(termios.TCIOFF, int)

assert isinstance(termios.TCION, int)

assert isinstance(termios.VTIME, int)

assert isinstance(termios.VMIN, int)

assert isinstance(termios.NCCS, int)

assert termios.VTIME < termios.NCCS

assert termios.VMIN < termios.NCCS
print("TestModule::test_constants: ok")
