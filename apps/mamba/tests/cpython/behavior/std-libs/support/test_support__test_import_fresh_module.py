# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_import_fresh_module"
# subject = "cpython.test_support.TestSupport.test_import_fresh_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_import_fresh_module
"""Auto-ported test: TestSupport::test_import_fresh_module (CPython 3.12 oracle)."""


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
import_helper.import_fresh_module('ftplib')
print("TestSupport::test_import_fresh_module: ok")
