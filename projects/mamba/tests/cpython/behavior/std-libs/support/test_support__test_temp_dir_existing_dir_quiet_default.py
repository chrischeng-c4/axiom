# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_dir_existing_dir_quiet_default"
# subject = "cpython.test_support.TestSupport.test_temp_dir__existing_dir__quiet_default"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_dir__existing_dir__quiet_default
"""Auto-ported test: TestSupport::test_temp_dir__existing_dir__quiet_default (CPython 3.12 oracle)."""


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
"""Test passing a directory that already exists."""

def call_temp_dir(path):
    with os_helper.temp_dir(path) as temp_path:
        raise Exception('should not get here')
path = tempfile.mkdtemp()
path = os.path.realpath(path)
try:

    assert os.path.isdir(path)

    try:
        call_temp_dir(path)
        raise AssertionError('expected FileExistsError')
    except FileExistsError:
        pass

    assert os.path.isdir(path)
finally:
    shutil.rmtree(path)
print("TestSupport::test_temp_dir__existing_dir__quiet_default: ok")
