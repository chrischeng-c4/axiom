# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_dir"
# subject = "cpython.test_support.TestSupport.test_temp_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_dir
"""Auto-ported test: TestSupport::test_temp_dir (CPython 3.12 oracle)."""


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
"""Test that temp_dir() creates and destroys its directory."""
parent_dir = tempfile.mkdtemp()
parent_dir = os.path.realpath(parent_dir)
try:
    path = os.path.join(parent_dir, 'temp')

    assert not os.path.isdir(path)
    with os_helper.temp_dir(path) as temp_path:

        assert temp_path == path

        assert os.path.isdir(path)

    assert not os.path.isdir(path)
finally:
    os_helper.rmtree(parent_dir)
print("TestSupport::test_temp_dir: ok")
