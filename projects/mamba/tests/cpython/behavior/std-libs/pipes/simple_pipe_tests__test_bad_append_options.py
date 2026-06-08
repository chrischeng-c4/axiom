# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__test_bad_append_options"
# subject = "cpython.test_pipes.SimplePipeTests.testBadAppendOptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pipes.py::SimplePipeTests::testBadAppendOptions
"""Auto-ported test: SimplePipeTests::testBadAppendOptions (CPython 3.12 oracle)."""


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

try:
    t.append(7, pipes.STDIN_STDOUT)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    t.append('boguscmd', 'xx')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    t.append('boguscmd', pipes.SOURCE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
t = pipes.Template()
t.append('boguscmd', pipes.SINK)

try:
    t.append('boguscmd', pipes.SINK)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
t = pipes.Template()

try:
    t.append('boguscmd $OUT', pipes.FILEIN_FILEOUT)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
t = pipes.Template()

try:
    t.append('boguscmd', pipes.FILEIN_STDOUT)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
t = pipes.Template()

try:
    t.append('boguscmd $IN', pipes.FILEIN_FILEOUT)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
t = pipes.Template()

try:
    t.append('boguscmd', pipes.STDIN_FILEOUT)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("SimplePipeTests::testBadAppendOptions: ok")
