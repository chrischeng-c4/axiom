# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_unlink"
# subject = "cpython.test_support.TestSupport.test_unlink"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_unlink
"""Auto-ported test: TestSupport::test_unlink (CPython 3.12 oracle)."""


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
with open(TESTFN, 'w', encoding='utf-8') as f:
    pass
os_helper.unlink(TESTFN)

assert not os.path.exists(TESTFN)
os_helper.unlink(TESTFN)
print("TestSupport::test_unlink: ok")
