# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_incomplete_frame_are_invisible__test_issue95818"
# subject = "cpython.test_frame.TestIncompleteFrameAreInvisible.test_issue95818"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_frame.py::TestIncompleteFrameAreInvisible::test_issue95818
"""Auto-ported test: TestIncompleteFrameAreInvisible::test_issue95818 (CPython 3.12 oracle)."""


import gc
import operator
import re
import sys
import textwrap
import threading
import types
import unittest
import weakref
from test import support
from test.support import threading_helper
from test.support.script_helper import assert_python_ok


try:
    import _testcapi
except ImportError:
    _testcapi = None


# --- test body ---
code = textwrap.dedent(f'\n            import gc\n\n            gc.set_threshold(1,1,1)\n            class GCHello:\n                def __del__(self):\n                    print("Destroyed from gc")\n\n            def gen():\n                yield\n\n            fd = open({__file__!r})\n            l = [fd, GCHello()]\n            l.append(l)\n            del fd\n            del l\n            gen()\n        ')
assert_python_ok('-c', code)
print("TestIncompleteFrameAreInvisible::test_issue95818: ok")
