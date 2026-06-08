# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_cwd"
# subject = "cpython.test_support.TestSupport.test_temp_cwd"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_cwd
"""Auto-ported test: TestSupport::test_temp_cwd (CPython 3.12 oracle)."""


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
here = os.getcwd()
with os_helper.temp_cwd(name=TESTFN):

    assert os.path.basename(os.getcwd()) == TESTFN

assert not os.path.exists(TESTFN)

assert os.getcwd() == here
print("TestSupport::test_temp_cwd: ok")
