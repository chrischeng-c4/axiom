# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_dir_forked_child"
# subject = "cpython.test_support.TestSupport.test_temp_dir__forked_child"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_dir__forked_child
"""Auto-ported test: TestSupport::test_temp_dir__forked_child (CPython 3.12 oracle)."""


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
"""Test that a forked child process does not remove the directory."""
script_helper.assert_python_ok('-c', textwrap.dedent('\n            import os\n            from test import support\n            from test.support import os_helper\n            with os_helper.temp_cwd() as temp_path:\n                pid = os.fork()\n                if pid != 0:\n                    # parent process\n\n                    # wait for the child to terminate\n                    support.wait_process(pid, exitcode=0)\n\n                    # Make sure that temp_path is still present. When the child\n                    # process leaves the \'temp_cwd\'-context, the __exit__()-\n                    # method of the context must not remove the temporary\n                    # directory.\n                    if not os.path.isdir(temp_path):\n                        raise AssertionError("Child removed temp_path.")\n        '))
print("TestSupport::test_temp_dir__forked_child: ok")
