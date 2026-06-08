# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_forget"
# subject = "cpython.test_support.TestSupport.test_forget"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_forget
"""Auto-ported test: TestSupport::test_forget (CPython 3.12 oracle)."""


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
mod_filename = TESTFN + '.py'
with open(mod_filename, 'w', encoding='utf-8') as f:
    print('foo = 1', file=f)
sys.path.insert(0, os.curdir)
importlib.invalidate_caches()
try:
    mod = __import__(TESTFN)

    assert TESTFN in sys.modules
    import_helper.forget(TESTFN)

    assert TESTFN not in sys.modules
finally:
    del sys.path[0]
    os_helper.unlink(mod_filename)
    os_helper.rmtree('__pycache__')
print("TestSupport::test_forget: ok")
