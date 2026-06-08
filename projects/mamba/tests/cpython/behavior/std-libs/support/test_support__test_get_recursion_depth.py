# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_get_recursion_depth"
# subject = "cpython.test_support.TestSupport.test_get_recursion_depth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_get_recursion_depth
"""Auto-ported test: TestSupport::test_get_recursion_depth (CPython 3.12 oracle)."""


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
code = textwrap.dedent('\n            from test import support\n            import sys\n\n            def check(cond):\n                if not cond:\n                    raise AssertionError("test failed")\n\n            # depth 1\n            check(support.get_recursion_depth() == 1)\n\n            # depth 2\n            def test_func():\n                check(support.get_recursion_depth() == 2)\n            test_func()\n\n            def test_recursive(depth, limit):\n                if depth >= limit:\n                    # cannot call get_recursion_depth() at this depth,\n                    # it can raise RecursionError\n                    return\n                get_depth = support.get_recursion_depth()\n                print(f"test_recursive: {depth}/{limit}: "\n                      f"get_recursion_depth() says {get_depth}")\n                check(get_depth == depth)\n                test_recursive(depth + 1, limit)\n\n            # depth up to 25\n            with support.infinite_recursion(max_depth=25):\n                limit = sys.getrecursionlimit()\n                print(f"test with sys.getrecursionlimit()={limit}")\n                test_recursive(2, limit)\n\n            # depth up to 500\n            with support.infinite_recursion(max_depth=500):\n                limit = sys.getrecursionlimit()\n                print(f"test with sys.getrecursionlimit()={limit}")\n                test_recursive(2, limit)\n        ')
script_helper.assert_python_ok('-c', code)
print("TestSupport::test_get_recursion_depth: ok")
