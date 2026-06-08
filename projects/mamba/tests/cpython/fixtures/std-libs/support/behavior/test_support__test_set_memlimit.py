# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_set_memlimit"
# subject = "cpython.test_support.TestSupport.test_set_memlimit"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_set_memlimit
"""Auto-ported test: TestSupport::test_set_memlimit (CPython 3.12 oracle)."""


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
_4GiB = 4 * 1024 ** 3
TiB = 1024 ** 4
old_max_memuse = support.max_memuse
old_real_max_memuse = support.real_max_memuse
try:
    if sys.maxsize > 2 ** 32:
        support.set_memlimit('4g')

        assert support.max_memuse == _4GiB

        assert support.real_max_memuse == _4GiB
        big = 2 ** 100 // TiB
        support.set_memlimit(f'{big}t')

        assert support.max_memuse == sys.maxsize

        assert support.real_max_memuse == big * TiB
    else:
        support.set_memlimit('4g')

        assert support.max_memuse == sys.maxsize

        assert support.real_max_memuse == _4GiB
finally:
    support.max_memuse = old_max_memuse
    support.real_max_memuse = old_real_max_memuse
print("TestSupport::test_set_memlimit: ok")
