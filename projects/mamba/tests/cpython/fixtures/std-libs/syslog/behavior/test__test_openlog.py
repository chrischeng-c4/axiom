# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "behavior"
# case = "test__test_openlog"
# subject = "cpython.test_syslog.Test.test_openlog"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_syslog.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_syslog.py::Test::test_openlog
"""Auto-ported test: Test::test_openlog (CPython 3.12 oracle)."""


from test.support import import_helper, threading_helper
from test import support
import sys
import threading
import time
import unittest
from textwrap import dedent


syslog = import_helper.import_module('syslog')


# --- test body ---
syslog.openlog('python')

try:
    syslog.openlog('\ud800')
    raise AssertionError('expected UnicodeEncodeError')
except UnicodeEncodeError:
    pass
print("Test::test_openlog: ok")
