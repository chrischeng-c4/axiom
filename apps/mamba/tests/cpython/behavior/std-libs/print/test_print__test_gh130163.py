# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "print"
# dimension = "behavior"
# case = "test_print__test_gh130163"
# subject = "cpython.test_print.TestPrint.test_gh130163"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_print.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_print.py::TestPrint::test_gh130163
"""Auto-ported test: TestPrint::test_gh130163 (CPython 3.12 oracle)."""


import unittest
import sys
from io import StringIO
from test import support


NotDefined = object()

dispatch = {(False, False, False): lambda args, sep, end, file: print(*args), (False, False, True): lambda args, sep, end, file: print(*args, file=file), (False, True, False): lambda args, sep, end, file: print(*args, end=end), (False, True, True): lambda args, sep, end, file: print(*args, end=end, file=file), (True, False, False): lambda args, sep, end, file: print(*args, sep=sep), (True, False, True): lambda args, sep, end, file: print(*args, sep=sep, file=file), (True, True, False): lambda args, sep, end, file: print(*args, sep=sep, end=end), (True, True, True): lambda args, sep, end, file: print(*args, sep=sep, end=end, file=file)}

class ClassWith__str__:

    def __init__(self, x):
        self.x = x

    def __str__(self):
        return self.x


# --- test body ---
class X:

    def __str__(self):
        sys.stdout = StringIO()
        support.gc_collect()
        return 'foo'
with support.swap_attr(sys, 'stdout', None):
    sys.stdout = StringIO()
    print(X())
print("TestPrint::test_gh130163: ok")
