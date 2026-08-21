# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "behavior"
# case = "test_module__test_exception"
# subject = "cpython.test_termios.TestModule.test_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_termios.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_termios.py::TestModule::test_exception
"""Auto-ported test: TestModule::test_exception (CPython 3.12 oracle)."""


import errno
import os
import sys
import tempfile
import unittest
from test.support.import_helper import import_module


termios = import_module('termios')


# --- test body ---

assert issubclass(termios.error, Exception)

assert not issubclass(termios.error, OSError)
print("TestModule::test_exception: ok")
