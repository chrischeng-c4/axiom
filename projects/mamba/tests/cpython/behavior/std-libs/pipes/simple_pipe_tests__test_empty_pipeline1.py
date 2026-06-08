# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__test_empty_pipeline1"
# subject = "cpython.test_pipes.SimplePipeTests.testEmptyPipeline1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pipes.py::SimplePipeTests::testEmptyPipeline1
"""Auto-ported test: SimplePipeTests::testEmptyPipeline1 (CPython 3.12 oracle)."""


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
d = 'empty pipeline test COPY'
with open(TESTFN, 'w') as f:
    f.write(d)
with open(TESTFN2, 'w') as f:
    f.write('')
t = pipes.Template()
t.copy(TESTFN, TESTFN2)
with open(TESTFN2) as f:

    assert f.read() == d
print("SimplePipeTests::testEmptyPipeline1: ok")
