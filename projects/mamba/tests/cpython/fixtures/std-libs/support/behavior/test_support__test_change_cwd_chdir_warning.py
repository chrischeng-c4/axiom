# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_change_cwd_chdir_warning"
# subject = "cpython.test_support.TestSupport.test_change_cwd__chdir_warning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_change_cwd__chdir_warning
"""Auto-ported test: TestSupport::test_change_cwd__chdir_warning (CPython 3.12 oracle)."""


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
"""Check the warning message when os.chdir() fails."""
path = TESTFN + '_does_not_exist'
with warnings_helper.check_warnings() as recorder:
    with os_helper.change_cwd(path=path, quiet=True):
        pass
    messages = [str(w.message) for w in recorder.warnings]

assert len(messages) == 1
msg = messages[0]

assert msg.startswith(f'tests may fail, unable to change the current working directory to {path!r}: ')
print("TestSupport::test_change_cwd__chdir_warning: ok")
