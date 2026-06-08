# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "print"
# dimension = "behavior"
# case = "test_print__test_print_flush"
# subject = "cpython.test_print.TestPrint.test_print_flush"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_print.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_print.py::TestPrint::test_print_flush
"""Auto-ported test: TestPrint::test_print_flush (CPython 3.12 oracle)."""


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
class filelike:

    def __init__(self):
        self.written = ''
        self.flushed = 0

    def write(self, str):
        self.written += str

    def flush(self):
        self.flushed += 1
f = filelike()
print(1, file=f, end='', flush=True)
print(2, file=f, end='', flush=True)
print(3, file=f, flush=False)

assert f.written == '123\n'

assert f.flushed == 2

class noflush:

    def write(self, str):
        pass

    def flush(self):
        raise RuntimeError

try:
    print(1, file=noflush(), flush=True)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("TestPrint::test_print_flush: ok")
