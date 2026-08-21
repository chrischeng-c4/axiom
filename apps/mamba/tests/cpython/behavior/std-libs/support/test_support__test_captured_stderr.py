# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_captured_stderr"
# subject = "cpython.test_support.TestSupport.test_captured_stderr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_captured_stderr
"""Auto-ported test: TestSupport::test_captured_stderr (CPython 3.12 oracle)."""


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
with support.captured_stderr() as stderr:
    print('hello', file=sys.stderr)

assert stderr.getvalue() == 'hello\n'
print("TestSupport::test_captured_stderr: ok")
