# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_import"
# subject = "cpython.test_compile.TestSpecifics.test_import"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_import
"""Auto-ported test: TestSpecifics::test_import (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
succeed = ['import sys', 'import os, sys', 'import os as bar', 'import os.path as bar', 'from __future__ import nested_scopes, generators', 'from __future__ import (nested_scopes,\ngenerators)', 'from __future__ import (nested_scopes,\ngenerators,)', 'from sys import stdin, stderr, stdout', 'from sys import (stdin, stderr,\nstdout)', 'from sys import (stdin, stderr,\nstdout,)', 'from sys import (stdin\n, stderr, stdout)', 'from sys import (stdin\n, stderr, stdout,)', 'from sys import stdin as si, stdout as so, stderr as se', 'from sys import (stdin as si, stdout as so, stderr as se)', 'from sys import (stdin as si, stdout as so, stderr as se,)']
fail = ['import (os, sys)', 'import (os), (sys)', 'import ((os), (sys))', 'import (sys', 'import sys)', 'import (os,)', 'import os As bar', 'import os.path a bar', 'from sys import stdin As stdout', 'from sys import stdin a stdout', 'from (sys) import stdin', 'from __future__ import (nested_scopes', 'from __future__ import nested_scopes)', 'from __future__ import nested_scopes,\ngenerators', 'from sys import (stdin', 'from sys import stdin)', 'from sys import stdin, stdout,\nstderr', 'from sys import stdin si', 'from sys import stdin,', 'from sys import (*)', 'from sys import (stdin,, stdout, stderr)', 'from sys import (stdin, stdout),']
for stmt in succeed:
    compile(stmt, 'tmp', 'exec')
for stmt in fail:

    try:
        compile(stmt, 'tmp', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
print("TestSpecifics::test_import: ok")
