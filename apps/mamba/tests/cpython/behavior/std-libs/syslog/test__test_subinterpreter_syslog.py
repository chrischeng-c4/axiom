# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "behavior"
# case = "test__test_subinterpreter_syslog"
# subject = "cpython.test_syslog.Test.test_subinterpreter_syslog"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_syslog.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_syslog.py::Test::test_subinterpreter_syslog
"""Auto-ported test: Test::test_subinterpreter_syslog (CPython 3.12 oracle)."""


from test.support import import_helper, threading_helper
from test import support
import sys
import threading
import time
import unittest
from textwrap import dedent


syslog = import_helper.import_module('syslog')


# --- test body ---
code = dedent("\n                import syslog\n                caught_error = False\n                try:\n                    syslog.syslog('foo')\n                except RuntimeError:\n                    caught_error = True\n                assert(caught_error)\n            ")
res = support.run_in_subinterp(code)

assert res == 0
syslog.openlog()
try:
    code = dedent("\n                    import syslog\n                    syslog.syslog('foo')\n                ")
    res = support.run_in_subinterp(code)

    assert res == 0
finally:
    syslog.closelog()
print("Test::test_subinterpreter_syslog: ok")
