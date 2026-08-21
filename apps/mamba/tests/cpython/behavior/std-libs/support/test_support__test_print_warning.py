# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_print_warning"
# subject = "cpython.test_support.TestSupport.test_print_warning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_print_warning
"""Auto-ported test: TestSupport::test_print_warning (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
def check_options(args, func, expected=None):
    code = f'from test.support import {func}; print(repr({func}()))'
    cmd = [sys.executable, *args, '-c', code]
    env = {key: value for key, value in os.environ.items() if not key.startswith('PYTHON')}
    proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, universal_newlines=True, env=env)
    if expected is None:
        expected = args

    assert proc.stdout.rstrip() == repr(expected)

    assert proc.returncode == 0

def check_print_warning(msg, expected):
    stderr = io.StringIO()
    with support.swap_attr(support.print_warning, 'orig_stderr', stderr):
        support.print_warning(msg)

    assert stderr.getvalue() == expected
check_print_warning('msg', 'Warning -- msg\n')
check_print_warning('a\nb', 'Warning -- a\nWarning -- b\n')
print("TestSupport::test_print_warning: ok")
