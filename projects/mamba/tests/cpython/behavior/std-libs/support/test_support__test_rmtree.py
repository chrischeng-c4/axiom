# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_rmtree"
# subject = "cpython.test_support.TestSupport.test_rmtree"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_rmtree
"""Auto-ported test: TestSupport::test_rmtree (CPython 3.12 oracle)."""


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
dirpath = os_helper.TESTFN + 'd'
subdirpath = os.path.join(dirpath, 'subdir')
os.mkdir(dirpath)
os.mkdir(subdirpath)
os_helper.rmtree(dirpath)

assert not os.path.exists(dirpath)
with support.swap_attr(support, 'verbose', 0):
    os_helper.rmtree(dirpath)
os.mkdir(dirpath)
os.mkdir(subdirpath)
os.chmod(dirpath, stat.S_IRUSR | stat.S_IXUSR)
with support.swap_attr(support, 'verbose', 0):
    os_helper.rmtree(dirpath)

assert not os.path.exists(dirpath)
os.mkdir(dirpath)
os.mkdir(subdirpath)
os.chmod(dirpath, 0)
with support.swap_attr(support, 'verbose', 0):
    os_helper.rmtree(dirpath)

assert not os.path.exists(dirpath)
print("TestSupport::test_rmtree: ok")
