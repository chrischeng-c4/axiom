# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "openpty"
# dimension = "behavior"
# case = "openpty_test__test"
# subject = "cpython.test_openpty.OpenptyTest.test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_openpty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_openpty.py::OpenptyTest::test
"""Auto-ported test: OpenptyTest::test (CPython 3.12 oracle)."""


import os, unittest


if not hasattr(os, 'openpty'):
    raise unittest.SkipTest('os.openpty() not available.')


# --- test body ---
master, slave = os.openpty()
pass
pass
if not os.isatty(slave):

    raise AssertionError('Slave-end of pty is not a terminal.')
os.write(slave, b'Ping!')

assert os.read(master, 1024) == b'Ping!'
print("OpenptyTest::test: ok")
