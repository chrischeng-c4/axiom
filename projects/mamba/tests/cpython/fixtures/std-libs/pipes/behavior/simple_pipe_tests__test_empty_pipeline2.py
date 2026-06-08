# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__test_empty_pipeline2"
# subject = "cpython.test_pipes.SimplePipeTests.testEmptyPipeline2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pipes.py::SimplePipeTests::testEmptyPipeline2
"""Auto-ported test: SimplePipeTests::testEmptyPipeline2 (CPython 3.12 oracle)."""


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
d = 'empty pipeline test READ'
with open(TESTFN, 'w') as f:
    f.write(d)
t = pipes.Template()
f = t.open(TESTFN, 'r')
try:

    assert f.read() == d
finally:
    f.close()
print("SimplePipeTests::testEmptyPipeline2: ok")
