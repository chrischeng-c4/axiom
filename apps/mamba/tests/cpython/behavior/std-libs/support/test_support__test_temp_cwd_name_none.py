# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_cwd_name_none"
# subject = "cpython.test_support.TestSupport.test_temp_cwd__name_none"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_cwd__name_none
"""Auto-ported test: TestSupport::test_temp_cwd__name_none (CPython 3.12 oracle)."""


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
"""Test passing None to temp_cwd()."""
original_cwd = os.getcwd()
with os_helper.temp_cwd(name=None) as new_cwd:

    assert new_cwd != original_cwd

    assert os.path.isdir(new_cwd)

    assert os.getcwd() == new_cwd

assert os.getcwd() == original_cwd
print("TestSupport::test_temp_cwd__name_none: ok")
