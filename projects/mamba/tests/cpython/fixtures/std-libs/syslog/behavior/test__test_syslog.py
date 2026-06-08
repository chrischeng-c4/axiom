# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "behavior"
# case = "test__test_syslog"
# subject = "cpython.test_syslog.Test.test_syslog"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_syslog.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_syslog.py::Test::test_syslog
"""Auto-ported test: Test::test_syslog (CPython 3.12 oracle)."""


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
syslog.syslog('test message from python test_syslog')
syslog.syslog(syslog.LOG_ERR, 'test error from python test_syslog')
print("Test::test_syslog: ok")
