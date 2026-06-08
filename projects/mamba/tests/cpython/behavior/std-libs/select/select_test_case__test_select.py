# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "behavior"
# case = "select_test_case__test_select"
# subject = "cpython.test_select.SelectTestCase.test_select"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_select.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_select.py::SelectTestCase::test_select
"""Auto-ported test: SelectTestCase::test_select (CPython 3.12 oracle)."""


import errno
import select
import subprocess
import sys
import textwrap
import unittest
from test import support


support.requires_working_socket(module=True)

def tearDownModule():
    support.reap_children()


# --- test body ---
code = textwrap.dedent('\n            import time\n            for i in range(10):\n                print("testing...", flush=True)\n                time.sleep(0.050)\n        ')
cmd = [sys.executable, '-I', '-c', code]
with subprocess.Popen(cmd, stdout=subprocess.PIPE) as proc:
    pipe = proc.stdout
    for timeout in (0, 1, 2, 4, 8, 16) + (None,) * 10:
        if support.verbose:
            print(f'timeout = {timeout}')
        rfd, wfd, xfd = select.select([pipe], [], [], timeout)

        assert wfd == []

        assert xfd == []
        if not rfd:
            continue
        if rfd == [pipe]:
            line = pipe.readline()
            if support.verbose:
                print(repr(line))
            if not line:
                if support.verbose:
                    print('EOF')
                break
            continue

        raise AssertionError('Unexpected return values from select():')
print("SelectTestCase::test_select: ok")
