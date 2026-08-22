use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/code/code_argcount_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_argcount_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_argcount_attrs"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co_argcount / co_posonlyargcount / co_kwonlyargcount reflect the signature shape of `def f(a, b, *, z=1, w=2)`: 2 / 0 / 2"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.co_argcount == 2, f"co_argcount = {co.co_argcount}"
assert co.co_posonlyargcount == 0, f"co_posonlyargcount = {co.co_posonlyargcount}"
assert co.co_kwonlyargcount == 2, f"co_kwonlyargcount = {co.co_kwonlyargcount}"

print("code_argcount_attrs OK")
"###);
    assert_output(&out, r###"code_argcount_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_constructor_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_constructor_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_constructor_roundtrip"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: rebuilding a CodeType from an existing code object's full field list (the 18-arg 3.12 constructor) preserves co_name and co_argcount"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
CodeType = type(co)
rebuilt = CodeType(
    co.co_argcount, co.co_posonlyargcount, co.co_kwonlyargcount,
    co.co_nlocals, co.co_stacksize, co.co_flags, co.co_code,
    co.co_consts, co.co_names, co.co_varnames, co.co_filename,
    co.co_name, co.co_qualname, co.co_firstlineno, co.co_linetable,
    co.co_exceptiontable, co.co_freevars, co.co_cellvars,
)
assert rebuilt.co_name == co.co_name, "constructor round-trip preserves name"
assert rebuilt.co_argcount == co.co_argcount, "round-trip preserves argcount"

print("code_constructor_roundtrip OK")
"###);
    assert_output(&out, r###"code_constructor_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_equality_on_replace.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_equality_on_replace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_equality_on_replace"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code equality: c.replace() with no changes compares equal to c, while c.replace(co_name=other) compares unequal"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.replace() == co, "no-op replace() compares equal"
assert co.replace(co_name="renamed") != co, "renaming makes the code object unequal"

print("code_equality_on_replace OK")
"###);
    assert_output(&out, r###"code_equality_on_replace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_hash_uses_firstlineno.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_hash_uses_firstlineno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_hash_uses_firstlineno"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code hashing follows equality: an equal replace() hashes the same, while changing co_firstlineno changes the hash"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert hash(co.replace()) == hash(co), "equal code objects hash the same"
shifted = co.replace(co_firstlineno=co.co_firstlineno + 1)
assert hash(shifted) != hash(co), "co_firstlineno is part of the hash"

print("code_hash_uses_firstlineno OK")
"###);
    assert_output(&out, r###"code_hash_uses_firstlineno OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_name_and_varnames_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_name_and_varnames_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_name_and_varnames_attrs"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co_name is the function name, a local ('x') appears in co_varnames, and co_nlocals is positive"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.co_name == "sample", f"co_name = {co.co_name!r}"
assert "x" in co.co_varnames, f"co_varnames = {co.co_varnames!r}"
assert co.co_nlocals > 0, f"co_nlocals = {co.co_nlocals}"

print("code_name_and_varnames_attrs OK")
"###);
    assert_output(&out, r###"code_name_and_varnames_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_replace_firstlineno_distinct.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_replace_firstlineno_distinct() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_replace_firstlineno_distinct"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: replacing co_firstlineno shifts the line number and yields a distinct, unequal code object"""
import types

c1 = (lambda: 1).__code__
c_shift = c1.replace(co_firstlineno=c1.co_firstlineno + 5)
assert c_shift.co_firstlineno == c1.co_firstlineno + 5, "firstlineno shifted by 5"
assert c1 != c_shift, "firstlineno change -> unequal code object"

print("code_replace_firstlineno_distinct OK")
"###);
    assert_output(&out, r###"code_replace_firstlineno_distinct OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_replace_preserves_unchanged.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_replace_preserves_unchanged() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_replace_preserves_unchanged"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co.replace(co_name=same) returns a new code object preserving the unchanged field (co_name stays equal)"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
nc = co.replace(co_name="sample")
assert nc is not co, "replace returns a new code object"
assert nc.co_name == "sample", f"replaced co_name = {nc.co_name!r}"
assert nc.co_argcount == co.co_argcount, "untouched field preserved"

print("code_replace_preserves_unchanged OK")
"###);
    assert_output(&out, r###"code_replace_preserves_unchanged OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_test__test_constructor.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_test__test_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_test__test_constructor"
# subject = "cpython.test_code.CodeTest.test_constructor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_code.py::CodeTest::test_constructor
"""Auto-ported test: CodeTest::test_constructor (CPython 3.12 oracle)."""


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
def func():
    pass
co = func.__code__
CodeType = type(co)
CodeType(co.co_argcount, co.co_posonlyargcount, co.co_kwonlyargcount, co.co_nlocals, co.co_stacksize, co.co_flags, co.co_code, co.co_consts, co.co_names, co.co_varnames, co.co_filename, co.co_name, co.co_qualname, co.co_firstlineno, co.co_linetable, co.co_exceptiontable, co.co_freevars, co.co_cellvars)
print("CodeTest::test_constructor: ok")
"###);
    assert_output(&out, r###"CodeTest::test_constructor: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_test__test_endline_and_columntable_none_when_no_debug_ranges.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_test__test_endline_and_columntable_none_when_no_debug_ranges() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"CodeTest::test_endline_and_columntable_none_when_no_debug_ranges: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_test__test_endline_and_columntable_none_when_no_debug_ranges_env.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_test__test_endline_and_columntable_none_when_no_debug_ranges_env() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_test__test_endline_and_columntable_none_when_no_debug_ranges_env"
# subject = "cpython.test_code.CodeTest.test_endline_and_columntable_none_when_no_debug_ranges_env"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_code.py::CodeTest::test_endline_and_columntable_none_when_no_debug_ranges_env
"""Auto-ported test: CodeTest::test_endline_and_columntable_none_when_no_debug_ranges_env (CPython 3.12 oracle)."""


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
assert_python_ok('-c', code, PYTHONNODEBUGRANGES='1')
print("CodeTest::test_endline_and_columntable_none_when_no_debug_ranges_env: ok")
"###);
    assert_output(&out, r###"CodeTest::test_endline_and_columntable_none_when_no_debug_ranges_env: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/code_type_is_codetype.py`.
#[test]
fn test_gen_behavior_std_libs_code_code_type_is_codetype() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_type_is_codetype"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: a function's __code__ attribute is a types.CodeType instance"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert type(co) is types.CodeType, f"code type = {type(co)!r}"

print("code_type_is_codetype OK")
"###);
    assert_output(&out, r###"code_type_is_codetype OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/compile_command_complete_returns_code.py`.
#[test]
fn test_gen_behavior_std_libs_code_compile_command_complete_returns_code() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "compile_command_complete_returns_code"
# subject = "code.compile_command"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command returns a types.CodeType object for complete source ('1 + 2'), not None"""
import code
import types

_cc = code.compile_command("1 + 2")
assert _cc is not None, "complete source compiles"
assert isinstance(_cc, types.CodeType), f"code object: {type(_cc)!r}"

print("compile_command_complete_returns_code OK")
"###);
    assert_output(&out, r###"compile_command_complete_returns_code OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/compile_command_incomplete_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_code_compile_command_incomplete_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "compile_command_incomplete_returns_none"
# subject = "code.compile_command"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command returns None for source that is syntactically incomplete and may still be continued ('if True:' / 'def f():')"""
import code

for src in ["if True:", "def f():"]:
    assert code.compile_command(src) is None, f"incomplete -> None: {src!r}"

print("compile_command_incomplete_returns_none OK")
"###);
    assert_output(&out, r###"compile_command_incomplete_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_locals_seed_visible.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_locals_seed_visible() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_locals_seed_visible"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: a fresh InteractiveConsole exposes its seeded locals dict via .locals, and the seeded value ('init_val' == 99) is readable"""
import code

_cons = code.InteractiveConsole({"init_val": 99})
assert isinstance(_cons.locals, dict), f"locals type = {type(_cons.locals)!r}"
assert _cons.locals.get("init_val") == 99, \
    f"locals['init_val'] = {_cons.locals.get('init_val')!r}"

print("console_locals_seed_visible OK")
"###);
    assert_output(&out, r###"console_locals_seed_visible OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_locals_shared_with_engine.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_locals_shared_with_engine() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_locals_shared_with_engine"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: the locals dict passed to InteractiveConsole is shared: a pushed statement reading a seeded name ('start') and writing a new one ('result') mutates that very dict"""
import code

_shared = {"start": 0}
_cons = code.InteractiveConsole(_shared)
_cons.push("result = start + 100")
assert _shared.get("result") == 100, f"shared locals: {_shared.get('result')!r}"

print("console_locals_shared_with_engine OK")
"###);
    assert_output(&out, r###"console_locals_shared_with_engine OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_push_false_for_complete.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_push_false_for_complete() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_push_false_for_complete"
# subject = "code.InteractiveConsole.push"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.push: InteractiveConsole.push returns False for complete statements: an assignment ('x = 42') and a bare expression ('1 + 1') each complete immediately"""
import code

_cons = code.InteractiveConsole({})
assert _cons.push("x = 42") is False, "assignment is a complete statement"
assert _cons.push("1 + 1") is False, "bare expression is a complete statement"

print("console_push_false_for_complete OK")
"###);
    assert_output(&out, r###"2
console_push_false_for_complete OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_push_true_then_blank_completes.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_push_true_then_blank_completes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_push_true_then_blank_completes"
# subject = "code.InteractiveConsole.push"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.push: push returns True while a compound statement is incomplete ('if True:'), accepts the indented body as a bool, and a final blank line completes the block (returns False)"""
import code

_cons = code.InteractiveConsole({})
assert _cons.push("if True:") is True, "compound header is incomplete"
# After the indented body push still returns a bool (more input may be wanted).
assert isinstance(_cons.push("    x = 1"), bool), "indented line returns bool"
# A blank line terminates the compound statement, completing the block.
assert _cons.push("") is False, "blank line completes the block"

print("console_push_true_then_blank_completes OK")
"###);
    assert_output(&out, r###"console_push_true_then_blank_completes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_resetbuffer_clears_partial.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_resetbuffer_clears_partial() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_resetbuffer_clears_partial"
# subject = "code.InteractiveConsole.resetbuffer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.resetbuffer: resetbuffer discards a partial multi-line statement so the next complete push ('y = 7') starts fresh and returns False"""
import code

_cons = code.InteractiveConsole({})
_cons.push("for i in range(10):")
_cons.resetbuffer()
# After reset the buffered 'for' header is gone, so a fresh complete statement
# completes immediately rather than being parsed as the loop body.
assert _cons.push("y = 7") is False, "fresh statement completes after reset"

print("console_resetbuffer_clears_partial OK")
"###);
    assert_output(&out, r###"console_resetbuffer_clears_partial OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_runsource_syntax_error_caught.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_runsource_syntax_error_caught() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_runsource_syntax_error_caught"
# subject = "code.InteractiveConsole.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.runsource: a genuine SyntaxError in source is caught and reported, not raised: ic.runsource('def bad(:') returns False with the error sent to showsyntaxerror"""
import code
import io
import contextlib

_cons = code.InteractiveConsole({})
_buf = io.StringIO()
with contextlib.redirect_stderr(_buf):
    _res = _cons.runsource("def bad(:")
# A real SyntaxError (not an incomplete continuation) is reported via
# showsyntaxerror and runsource returns False rather than propagating.
assert _res is False, f"syntax error -> False, got {_res!r}"
assert "SyntaxError" in _buf.getvalue(), "SyntaxError reported on stderr"

print("console_runsource_syntax_error_caught OK")
"###);
    assert_output(&out, r###"console_runsource_syntax_error_caught OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/console_subclasses_interpreter.py`.
#[test]
fn test_gen_behavior_std_libs_code_console_subclasses_interpreter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_subclasses_interpreter"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: InteractiveConsole is a subclass of InteractiveInterpreter, so an instance isinstance-checks as both"""
import code

assert issubclass(code.InteractiveConsole, code.InteractiveInterpreter), \
    "InteractiveConsole subclasses InteractiveInterpreter"
_cons = code.InteractiveConsole({"y": 99})
assert isinstance(_cons, code.InteractiveConsole), "instance is a Console"
assert isinstance(_cons, code.InteractiveInterpreter), "instance is also an Interpreter"

print("console_subclasses_interpreter OK")
"###);
    assert_output(&out, r###"console_subclasses_interpreter OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/interpreter_exec_into_namespace.py`.
#[test]
fn test_gen_behavior_std_libs_code_interpreter_exec_into_namespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "interpreter_exec_into_namespace"
# subject = "code.InteractiveInterpreter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter: an interpreter built over an explicit namespace dict execs source into it: after runsource('x = 42') the namespace holds x == 42"""
import code
import builtins

_ns = {"__builtins__": builtins}
_interp = code.InteractiveInterpreter(_ns)
_interp.runsource("x = 42")
assert _ns.get("x") == 42, f"namespace x = {_ns.get('x')!r}"

print("interpreter_exec_into_namespace OK")
"###);
    assert_output(&out, r###"interpreter_exec_into_namespace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/runsource_exec_error_caught_to_stderr.py`.
#[test]
fn test_gen_behavior_std_libs_code_runsource_exec_error_caught_to_stderr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_exec_error_caught_to_stderr"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: an exception from executed source is caught (not propagated): runsource('1 / 0') returns False and the traceback (containing 'ZeroDivisionError') is written to stderr via showtraceback"""
import code
import io
import contextlib

_interp = code.InteractiveInterpreter({})
_buf = io.StringIO()
with contextlib.redirect_stderr(_buf):
    _res = _interp.runsource("1 / 0")
# The runtime error is caught and reported, not raised; runsource still returns
# False (the source was complete) and the traceback is on stderr.
assert _res is False, f"complete-but-erroring source -> False, got {_res!r}"
assert "ZeroDivisionError" in _buf.getvalue(), "ZeroDivisionError traceback on stderr"

print("runsource_exec_error_caught_to_stderr OK")
"###);
    assert_output(&out, r###"runsource_exec_error_caught_to_stderr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/runsource_false_complete_true_incomplete.py`.
#[test]
fn test_gen_behavior_std_libs_code_runsource_false_complete_true_incomplete() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_false_complete_true_incomplete"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: runsource returns False for complete source ('z = 10') and True for incomplete source ('def foo():') that needs more input"""
import code

_interp = code.InteractiveInterpreter({})
assert _interp.runsource("z = 10") is False, "complete source -> False"
assert _interp.runsource("def foo():") is True, "incomplete source -> True (more input)"

print("runsource_false_complete_true_incomplete OK")
"###);
    assert_output(&out, r###"runsource_false_complete_true_incomplete OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/code/runsource_updates_locals.py`.
#[test]
fn test_gen_behavior_std_libs_code_runsource_updates_locals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_updates_locals"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: InteractiveInterpreter.runsource executes complete source and the result lands in the supplied locals dict: 'answer = 6 * 7' sets answer == 42"""
import code

_local = {}
_interp = code.InteractiveInterpreter(_local)
_interp.runsource("answer = 6 * 7")
assert _local.get("answer") == 42, f"runsource set local: {_local.get('answer')!r}"

print("runsource_updates_locals OK")
"###);
    assert_output(&out, r###"runsource_updates_locals OK
"###);
}
