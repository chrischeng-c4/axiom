# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_recursion"
# subject = "cpython.test_support.TestSupport.test_recursion"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_recursion
"""Auto-ported test: TestSupport::test_recursion (CPython 3.12 oracle)."""


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
def recursive_function(depth):
    if depth:
        recursive_function(depth - 1)
for max_depth in (5, 25, 250):
    with support.infinite_recursion(max_depth):
        available = support.get_recursion_available()
        recursive_function(available)
        try:
            recursive_function(available + 1)
        except RecursionError:
            pass
        else:

            raise AssertionError('RecursionError was not raised')
with support.infinite_recursion(3):
    try:
        recursive_function(3)
    except RecursionError:
        pass
    else:

        raise AssertionError('RecursionError was not raised')
print("TestSupport::test_recursion: ok")
