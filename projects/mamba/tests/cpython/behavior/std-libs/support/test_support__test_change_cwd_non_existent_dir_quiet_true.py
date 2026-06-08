# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_change_cwd_non_existent_dir_quiet_true"
# subject = "cpython.test_support.TestSupport.test_change_cwd__non_existent_dir__quiet_true"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_change_cwd__non_existent_dir__quiet_true
"""Auto-ported test: TestSupport::test_change_cwd__non_existent_dir__quiet_true (CPython 3.12 oracle)."""


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
"""Test passing a non-existent directory with quiet=True."""
original_cwd = os.getcwd()
with os_helper.temp_dir() as parent_dir:
    bad_dir = os.path.join(parent_dir, 'does_not_exist')
    with warnings_helper.check_warnings() as recorder:
        with os_helper.change_cwd(bad_dir, quiet=True) as new_cwd:

            assert new_cwd == original_cwd

            assert os.getcwd() == new_cwd
        warnings = [str(w.message) for w in recorder.warnings]

assert len(warnings) == 1
warn = warnings[0]

assert warn.startswith(f'tests may fail, unable to change the current working directory to {bad_dir!r}: ')
print("TestSupport::test_change_cwd__non_existent_dir__quiet_true: ok")
