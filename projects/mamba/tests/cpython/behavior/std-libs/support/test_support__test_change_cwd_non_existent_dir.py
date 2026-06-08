# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_change_cwd_non_existent_dir"
# subject = "cpython.test_support.TestSupport.test_change_cwd__non_existent_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_change_cwd__non_existent_dir
"""Auto-ported test: TestSupport::test_change_cwd__non_existent_dir (CPython 3.12 oracle)."""


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
"""Test passing a non-existent directory."""
original_cwd = os.getcwd()

def call_change_cwd(path):
    with os_helper.change_cwd(path) as new_cwd:
        raise Exception('should not get here')
with os_helper.temp_dir() as parent_dir:
    non_existent_dir = os.path.join(parent_dir, 'does_not_exist')

    try:
        call_change_cwd(non_existent_dir)
        raise AssertionError('expected FileNotFoundError')
    except FileNotFoundError:
        pass

assert os.getcwd() == original_cwd
print("TestSupport::test_change_cwd__non_existent_dir: ok")
