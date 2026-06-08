# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_swap_attr"
# subject = "cpython.test_support.TestSupport.test_swap_attr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_swap_attr
"""Auto-ported test: TestSupport::test_swap_attr (CPython 3.12 oracle)."""


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
class Obj:
    pass
obj = Obj()
obj.x = 1
with support.swap_attr(obj, 'x', 5) as x:

    assert obj.x == 5

    assert x == 1

assert obj.x == 1
with support.swap_attr(obj, 'y', 5) as y:

    assert obj.y == 5

    assert y is None

assert not hasattr(obj, 'y')
with support.swap_attr(obj, 'y', 5):
    del obj.y

assert not hasattr(obj, 'y')
print("TestSupport::test_swap_attr: ok")
