# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_dirs_on_sys_path"
# subject = "cpython.test_support.TestSupport.test_DirsOnSysPath"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_DirsOnSysPath
"""Auto-ported test: TestSupport::test_DirsOnSysPath (CPython 3.12 oracle)."""


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
with import_helper.DirsOnSysPath('foo', 'bar'):

    assert 'foo' in sys.path

    assert 'bar' in sys.path

assert 'foo' not in sys.path

assert 'bar' not in sys.path
print("TestSupport::test_DirsOnSysPath: ok")
