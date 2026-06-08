# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_test__test_endline_and_columntable_none_when_no_debug_ranges"
# subject = "cpython.test_code.CodeTest.test_endline_and_columntable_none_when_no_debug_ranges"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_code.py::CodeTest::test_endline_and_columntable_none_when_no_debug_ranges
"""Auto-ported test: CodeTest::test_endline_and_columntable_none_when_no_debug_ranges (CPython 3.12 oracle)."""


import inspect
import sys
import threading
import doctest
import unittest
import textwrap
import weakref
import dis
from test.support import cpython_only, check_impl_detail, requires_debug_ranges, gc_collect
from test.support.script_helper import assert_python_ok
from test.support import threading_helper
from opcode import opmap, opname


'This module includes tests of the code object representation.\n\n>>> def f(x):\n...     def g(y):\n...         return x + y\n...     return g\n...\n\n>>> dump(f.__code__)\nname: f\nargcount: 1\nposonlyargcount: 0\nkwonlyargcount: 0\nnames: ()\nvarnames: (\'x\', \'g\')\ncellvars: (\'x\',)\nfreevars: ()\nnlocals: 2\nflags: 3\nconsts: (\'None\', \'<code object g>\')\n\n>>> dump(f(4).__code__)\nname: g\nargcount: 1\nposonlyargcount: 0\nkwonlyargcount: 0\nnames: ()\nvarnames: (\'y\',)\ncellvars: ()\nfreevars: (\'x\',)\nnlocals: 1\nflags: 19\nconsts: (\'None\',)\n\n>>> def h(x, y):\n...     a = x + y\n...     b = x - y\n...     c = a * b\n...     return c\n...\n\n>>> dump(h.__code__)\nname: h\nargcount: 2\nposonlyargcount: 0\nkwonlyargcount: 0\nnames: ()\nvarnames: (\'x\', \'y\', \'a\', \'b\', \'c\')\ncellvars: ()\nfreevars: ()\nnlocals: 5\nflags: 3\nconsts: (\'None\',)\n\n>>> def attrs(obj):\n...     print(obj.attr1)\n...     print(obj.attr2)\n...     print(obj.attr3)\n\n>>> dump(attrs.__code__)\nname: attrs\nargcount: 1\nposonlyargcount: 0\nkwonlyargcount: 0\nnames: (\'print\', \'attr1\', \'attr2\', \'attr3\')\nvarnames: (\'obj\',)\ncellvars: ()\nfreevars: ()\nnlocals: 1\nflags: 3\nconsts: (\'None\',)\n\n>>> def optimize_away():\n...     \'doc string\'\n...     \'not a docstring\'\n...     53\n...     0x53\n\n>>> dump(optimize_away.__code__)\nname: optimize_away\nargcount: 0\nposonlyargcount: 0\nkwonlyargcount: 0\nnames: ()\nvarnames: ()\ncellvars: ()\nfreevars: ()\nnlocals: 0\nflags: 3\nconsts: ("\'doc string\'", \'None\')\n\n>>> def keywordonly_args(a,b,*,k1):\n...     return a,b,k1\n...\n\n>>> dump(keywordonly_args.__code__)\nname: keywordonly_args\nargcount: 2\nposonlyargcount: 0\nkwonlyargcount: 1\nnames: ()\nvarnames: (\'a\', \'b\', \'k1\')\ncellvars: ()\nfreevars: ()\nnlocals: 3\nflags: 3\nconsts: (\'None\',)\n\n>>> def posonly_args(a,b,/,c):\n...     return a,b,c\n...\n\n>>> dump(posonly_args.__code__)\nname: posonly_args\nargcount: 3\nposonlyargcount: 2\nkwonlyargcount: 0\nnames: ()\nvarnames: (\'a\', \'b\', \'c\')\ncellvars: ()\nfreevars: ()\nnlocals: 3\nflags: 3\nconsts: (\'None\',)\n\n'

try:
    import ctypes
except ImportError:
    ctypes = None

COPY_FREE_VARS = opmap['COPY_FREE_VARS']

def consts(t):
    """Yield a doctest-safe sequence of object reprs."""
    for elt in t:
        r = repr(elt)
        if r.startswith('<code object'):
            yield ('<code object %s>' % elt.co_name)
        else:
            yield r

def dump(co):
    """Print out a text representation of a code object."""
    for attr in ['name', 'argcount', 'posonlyargcount', 'kwonlyargcount', 'names', 'varnames', 'cellvars', 'freevars', 'nlocals', 'flags']:
        print('%s: %s' % (attr, getattr(co, 'co_' + attr)))
    print('consts:', tuple(consts(co.co_consts)))

def external_getitem(self, i):
    return f'Foreign getitem: {super().__getitem__(i)}'

def isinterned(s):
    return s is sys.intern(('_' + s + '_')[1:-1])

def read(it):
    return next(it)

def read_varint(it):
    b = read(it)
    val = b & 63
    shift = 0
    while b & 64:
        b = read(it)
        shift += 6
        val |= (b & 63) << shift
    return val

def read_signed_varint(it):
    uval = read_varint(it)
    if uval & 1:
        return -(uval >> 1)
    else:
        return uval >> 1

def parse_location_table(code):
    line = code.co_firstlineno
    it = iter(code.co_linetable)
    while True:
        try:
            first_byte = read(it)
        except StopIteration:
            return
        code = first_byte >> 3 & 15
        length = (first_byte & 7) + 1
        if code == 15:
            yield (code, length, None, None, None, None)
        elif code == 14:
            line_delta = read_signed_varint(it)
            line += line_delta
            end_line = line + read_varint(it)
            col = read_varint(it)
            if col == 0:
                col = None
            else:
                col -= 1
            end_col = read_varint(it)
            if end_col == 0:
                end_col = None
            else:
                end_col -= 1
            yield (code, length, line, end_line, col, end_col)
        elif code == 13:
            line_delta = read_signed_varint(it)
            line += line_delta
            yield (code, length, line, line, None, None)
        elif code in (10, 11, 12):
            line_delta = code - 10
            line += line_delta
            column = read(it)
            end_column = read(it)
            yield (code, length, line, line, column, end_column)
        else:
            assert 0 <= code < 10
            second_byte = read(it)
            column = code << 3 | second_byte >> 4
            yield (code, length, line, line, column, column + (second_byte & 15))

def positions_from_location_table(code):
    for _, length, line, end_line, col, end_col in parse_location_table(code):
        for _ in range(length):
            yield (line, end_line, col, end_col)

def dedup(lst, prev=object()):
    for item in lst:
        if item != prev:
            yield item
            prev = item

def lines_from_postions(positions):
    return dedup((l for l, _, _, _ in positions))

def misshappen():
    """





    """
    x = 4 + y
    y = a + b + d
    return q if x else p

def bug93662():
    example_report_generation_message = '\n            '.strip()
    raise ValueError()

if check_impl_detail(cpython=True) and ctypes is not None:
    py = ctypes.pythonapi
    freefunc = ctypes.CFUNCTYPE(None, ctypes.c_voidp)
    RequestCodeExtraIndex = py.PyUnstable_Eval_RequestCodeExtraIndex
    RequestCodeExtraIndex.argtypes = (freefunc,)
    RequestCodeExtraIndex.restype = ctypes.c_ssize_t
    SetExtra = py.PyUnstable_Code_SetExtra
    SetExtra.argtypes = (ctypes.py_object, ctypes.c_ssize_t, ctypes.c_voidp)
    SetExtra.restype = ctypes.c_int
    GetExtra = py.PyUnstable_Code_GetExtra
    GetExtra.argtypes = (ctypes.py_object, ctypes.c_ssize_t, ctypes.POINTER(ctypes.c_voidp))
    GetExtra.restype = ctypes.c_int
    LAST_FREED = None

    def myfree(ptr):
        global LAST_FREED
        LAST_FREED = ptr
    FREE_FUNC = freefunc(myfree)
    FREE_INDEX = RequestCodeExtraIndex(FREE_FUNC)

    class CoExtra(unittest.TestCase):

        def get_func(self):
            return eval('lambda:42')

        def test_get_non_code(self):
            f = self.get_func()
            self.assertRaises(SystemError, SetExtra, 42, FREE_INDEX, ctypes.c_voidp(100))
            self.assertRaises(SystemError, GetExtra, 42, FREE_INDEX, ctypes.c_voidp(100))

        def test_bad_index(self):
            f = self.get_func()
            self.assertRaises(SystemError, SetExtra, f.__code__, FREE_INDEX + 100, ctypes.c_voidp(100))
            self.assertEqual(GetExtra(f.__code__, FREE_INDEX + 100, ctypes.c_voidp(100)), 0)

        def test_free_called(self):
            f = self.get_func()
            SetExtra(f.__code__, FREE_INDEX, ctypes.c_voidp(100))
            del f
            self.assertEqual(LAST_FREED, 100)

        def test_get_set(self):
            f = self.get_func()
            extra = ctypes.c_voidp()
            SetExtra(f.__code__, FREE_INDEX, ctypes.c_voidp(200))
            SetExtra(f.__code__, FREE_INDEX, ctypes.c_voidp(300))
            self.assertEqual(LAST_FREED, 200)
            extra = ctypes.c_voidp()
            GetExtra(f.__code__, FREE_INDEX, extra)
            self.assertEqual(extra.value, 300)
            del f

        @threading_helper.requires_working_threading()
        def test_free_different_thread(self):
            f = self.get_func()

            class ThreadTest(threading.Thread):

                def __init__(self, f, test):
                    super().__init__()
                    self.f = f
                    self.test = test

                def run(self):
                    del self.f
                    self.test.assertEqual(LAST_FREED, 500)
            SetExtra(f.__code__, FREE_INDEX, ctypes.c_voidp(500))
            tt = ThreadTest(f, self)
            del f
            tt.start()
            tt.join()
            self.assertEqual(LAST_FREED, 500)

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite())
    return tests


# --- test body ---
code = textwrap.dedent('\n            def f():\n                pass\n\n            positions = f.__code__.co_positions()\n            for line, end_line, column, end_column in positions:\n                assert line == end_line\n                assert column is None\n                assert end_column is None\n            ')
assert_python_ok('-X', 'no_debug_ranges', '-c', code)
print("CodeTest::test_endline_and_columntable_none_when_no_debug_ranges: ok")
