# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__test_set_debug"
# subject = "cpython.test_pipes.SimplePipeTests.testSetDebug"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pipes.py::SimplePipeTests::testSetDebug
"""Auto-ported test: SimplePipeTests::testSetDebug (CPython 3.12 oracle)."""


import os
import string
import unittest
import shutil
from test.support import reap_children, unix_shell
from test.support.os_helper import TESTFN, unlink
from test.support.warnings_helper import import_deprecated


pipes = import_deprecated('pipes')

if os.name != 'posix':
    raise unittest.SkipTest('pipes module only works on posix')

if not (unix_shell and os.path.exists(unix_shell)):
    raise unittest.SkipTest('pipes module requires a shell')

TESTFN2 = TESTFN + '2'

s_command = 'tr %s %s' % (string.ascii_lowercase, string.ascii_uppercase)

def tearDownModule():
    reap_children()


# --- test body ---
t = pipes.Template()
t.debug(False)

assert t.debugging == False
t.debug(True)

assert t.debugging == True
print("SimplePipeTests::testSetDebug: ok")
