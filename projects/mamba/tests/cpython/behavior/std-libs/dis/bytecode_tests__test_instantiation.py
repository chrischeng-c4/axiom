# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "bytecode_tests__test_instantiation"
# subject = "cpython.test_dis.BytecodeTests.test_instantiation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dis.py::BytecodeTests::test_instantiation
"""Auto-ported test: BytecodeTests::test_instantiation (CPython 3.12 oracle)."""


import contextlib
import dis
import io
import opcode
import re
import sys
import tempfile
import types
import unittest
from test.support import captured_stdout, requires_debug_ranges, requires_specialization, cpython_only, os_helper
from test.support.bytecode_helper import BytecodeTestCase


def get_tb():

    def _error():
        try:
            1 / 0
        except Exception as e:
            tb = e.__traceback__
        return tb
    tb = _error()
    while tb.tb_next:
        tb = tb.tb_next
    return tb

TRACEBACK_CODE = get_tb().tb_frame.f_code

class _C:

    def __init__(self, x):
        self.x = x == 1

    @staticmethod
    def sm(x):
        x = x == 1

    @classmethod
    def cm(cls, x):
        cls.x = x == 1

dis_c_instance_method = '%3d        RESUME                   0\n\n%3d        LOAD_FAST                1 (x)\n           LOAD_CONST               1 (1)\n           COMPARE_OP              40 (==)\n           LOAD_FAST                0 (self)\n           STORE_ATTR               0 (x)\n           RETURN_CONST             0 (None)\n' % (_C.__init__.__code__.co_firstlineno, _C.__init__.__code__.co_firstlineno + 1)

dis_c_instance_method_bytes = '       RESUME                   0\n       LOAD_FAST                1\n       LOAD_CONST               1\n       COMPARE_OP              40 (==)\n       LOAD_FAST                0\n       STORE_ATTR               0\n       RETURN_CONST             0\n'

dis_c_class_method = '%3d        RESUME                   0\n\n%3d        LOAD_FAST                1 (x)\n           LOAD_CONST               1 (1)\n           COMPARE_OP              40 (==)\n           LOAD_FAST                0 (cls)\n           STORE_ATTR               0 (x)\n           RETURN_CONST             0 (None)\n' % (_C.cm.__code__.co_firstlineno, _C.cm.__code__.co_firstlineno + 2)

dis_c_static_method = '%3d        RESUME                   0\n\n%3d        LOAD_FAST                0 (x)\n           LOAD_CONST               1 (1)\n           COMPARE_OP              40 (==)\n           STORE_FAST               0 (x)\n           RETURN_CONST             0 (None)\n' % (_C.sm.__code__.co_firstlineno, _C.sm.__code__.co_firstlineno + 2)

dis_c = 'Disassembly of %s:\n%s\nDisassembly of %s:\n%s\nDisassembly of %s:\n%s\n' % (_C.__init__.__name__, dis_c_instance_method, _C.cm.__name__, dis_c_class_method, _C.sm.__name__, dis_c_static_method)

def _f(a):
    print(a)
    return 1

dis_f = '%3d        RESUME                   0\n\n%3d        LOAD_GLOBAL              1 (NULL + print)\n           LOAD_FAST                0 (a)\n           CALL                     1\n           POP_TOP\n\n%3d        RETURN_CONST             1 (1)\n' % (_f.__code__.co_firstlineno, _f.__code__.co_firstlineno + 1, _f.__code__.co_firstlineno + 2)

dis_f_co_code = '       RESUME                   0\n       LOAD_GLOBAL              1\n       LOAD_FAST                0\n       CALL                     1\n       POP_TOP\n       RETURN_CONST             1\n'

def bug708901():
    for res in range(1, 10):
        pass

dis_bug708901 = '%3d        RESUME                   0\n\n%3d        LOAD_GLOBAL              1 (NULL + range)\n           LOAD_CONST               1 (1)\n\n%3d        LOAD_CONST               2 (10)\n\n%3d        CALL                     2\n           GET_ITER\n        >> FOR_ITER                 2 (to 34)\n           STORE_FAST               0 (res)\n\n%3d        JUMP_BACKWARD            4 (to 26)\n\n%3d     >> END_FOR\n           RETURN_CONST             0 (None)\n' % (bug708901.__code__.co_firstlineno, bug708901.__code__.co_firstlineno + 1, bug708901.__code__.co_firstlineno + 2, bug708901.__code__.co_firstlineno + 1, bug708901.__code__.co_firstlineno + 3, bug708901.__code__.co_firstlineno + 1)

def bug1333982(x=[]):
    assert 0, (s for s in x) + 1
    pass

dis_bug1333982 = '%3d        RESUME                   0\n\n%3d        LOAD_ASSERTION_ERROR\n           LOAD_CONST               1 (<code object <genexpr> at 0x..., file "%s", line %d>)\n           MAKE_FUNCTION            0\n           LOAD_FAST                0 (x)\n           GET_ITER\n           CALL                     0\n\n%3d        LOAD_CONST               2 (1)\n\n%3d        BINARY_OP                0 (+)\n           CALL                     0\n           RAISE_VARARGS            1\n' % (bug1333982.__code__.co_firstlineno, bug1333982.__code__.co_firstlineno + 1, __file__, bug1333982.__code__.co_firstlineno + 1, bug1333982.__code__.co_firstlineno + 2, bug1333982.__code__.co_firstlineno + 1)

def bug42562():
    pass

bug42562.__code__ = bug42562.__code__.replace(co_linetable=b'\xf8')

dis_bug42562 = '       RESUME                   0\n       RETURN_CONST             0 (None)\n'

code_bug_45757 = bytes([144, 1, 9, 255, 144, 1, 100, 41, 83, 0])

dis_bug_45757 = '       EXTENDED_ARG             1\n       NOP\n       EXTENDED_ARG             1\n       LOAD_CONST             297\n       RETURN_VALUE\n'

bug46724 = bytes([opcode.EXTENDED_ARG, 255, opcode.EXTENDED_ARG, 255, opcode.EXTENDED_ARG, 255, opcode.opmap['JUMP_FORWARD'], 252])

dis_bug46724 = '    >> EXTENDED_ARG           255\n       EXTENDED_ARG         65535\n       EXTENDED_ARG         16777215\n       JUMP_FORWARD            -4 (to 0)\n'

def func_w_kwargs(a, b, **c):
    pass

def wrap_func_w_kwargs():
    func_w_kwargs(1, 2, c=5)

dis_kw_names = "%3d        RESUME                   0\n\n%3d        LOAD_GLOBAL              1 (NULL + func_w_kwargs)\n           LOAD_CONST               1 (1)\n           LOAD_CONST               2 (2)\n           LOAD_CONST               3 (5)\n           KW_NAMES                 4 (('c',))\n           CALL                     3\n           POP_TOP\n           RETURN_CONST             0 (None)\n" % (wrap_func_w_kwargs.__code__.co_firstlineno, wrap_func_w_kwargs.__code__.co_firstlineno + 1)

dis_intrinsic_1_2 = "  0        RESUME                   0\n\n  1        LOAD_CONST               0 (0)\n           LOAD_CONST               1 (('*',))\n           IMPORT_NAME              0 (math)\n           CALL_INTRINSIC_1         2 (INTRINSIC_IMPORT_STAR)\n           POP_TOP\n           RETURN_CONST             2 (None)\n"

dis_intrinsic_1_5 = '  0        RESUME                   0\n\n  1        LOAD_NAME                0 (a)\n           CALL_INTRINSIC_1         5 (INTRINSIC_UNARY_POSITIVE)\n           RETURN_VALUE\n'

dis_intrinsic_1_6 = '  0        RESUME                   0\n\n  1        BUILD_LIST               0\n           LOAD_NAME                0 (a)\n           LIST_EXTEND              1\n           CALL_INTRINSIC_1         6 (INTRINSIC_LIST_TO_TUPLE)\n           RETURN_VALUE\n'

_BIG_LINENO_FORMAT = '  1        RESUME                   0\n\n%3d        LOAD_GLOBAL              0 (spam)\n           POP_TOP\n           RETURN_CONST             0 (None)\n'

_BIG_LINENO_FORMAT2 = '   1        RESUME                   0\n\n%4d        LOAD_GLOBAL              0 (spam)\n            POP_TOP\n            RETURN_CONST             0 (None)\n'

dis_module_expected_results = 'Disassembly of f:\n  4        RESUME                   0\n           RETURN_CONST             0 (None)\n\nDisassembly of g:\n  5        RESUME                   0\n           RETURN_CONST             0 (None)\n\n'

expr_str = 'x + 1'

dis_expr_str = '  0        RESUME                   0\n\n  1        LOAD_NAME                0 (x)\n           LOAD_CONST               0 (1)\n           BINARY_OP                0 (+)\n           RETURN_VALUE\n'

simple_stmt_str = 'x = x + 1'

dis_simple_stmt_str = '  0        RESUME                   0\n\n  1        LOAD_NAME                0 (x)\n           LOAD_CONST               0 (1)\n           BINARY_OP                0 (+)\n           STORE_NAME               0 (x)\n           RETURN_CONST             1 (None)\n'

annot_stmt_str = '\nx: int = 1\ny: fun(1)\nlst[fun(0)]: int = 1\n'

dis_annot_stmt_str = "  0        RESUME                   0\n\n  2        SETUP_ANNOTATIONS\n           LOAD_CONST               0 (1)\n           STORE_NAME               0 (x)\n           LOAD_NAME                1 (int)\n           LOAD_NAME                2 (__annotations__)\n           LOAD_CONST               1 ('x')\n           STORE_SUBSCR\n\n  3        PUSH_NULL\n           LOAD_NAME                3 (fun)\n           LOAD_CONST               0 (1)\n           CALL                     1\n           LOAD_NAME                2 (__annotations__)\n           LOAD_CONST               2 ('y')\n           STORE_SUBSCR\n\n  4        LOAD_CONST               0 (1)\n           LOAD_NAME                4 (lst)\n           PUSH_NULL\n           LOAD_NAME                3 (fun)\n           LOAD_CONST               3 (0)\n           CALL                     1\n           STORE_SUBSCR\n           LOAD_NAME                1 (int)\n           POP_TOP\n           RETURN_CONST             4 (None)\n"

compound_stmt_str = 'x = 0\nwhile 1:\n    x += 1'

dis_compound_stmt_str = '  0        RESUME                   0\n\n  1        LOAD_CONST               0 (0)\n           STORE_NAME               0 (x)\n\n  2        NOP\n\n  3     >> LOAD_NAME                0 (x)\n           LOAD_CONST               1 (1)\n           BINARY_OP               13 (+=)\n           STORE_NAME               0 (x)\n\n  2        JUMP_BACKWARD            6 (to 8)\n'

dis_traceback = '%3d        RESUME                   0\n\n%3d        NOP\n\n%3d        LOAD_CONST               1 (1)\n           LOAD_CONST               2 (0)\n    -->    BINARY_OP               11 (/)\n           POP_TOP\n\n%3d        LOAD_FAST_CHECK          1 (tb)\n           RETURN_VALUE\n        >> PUSH_EXC_INFO\n\n%3d        LOAD_GLOBAL              0 (Exception)\n           CHECK_EXC_MATCH\n           POP_JUMP_IF_FALSE       23 (to 80)\n           STORE_FAST               0 (e)\n\n%3d        LOAD_FAST                0 (e)\n           LOAD_ATTR                2 (__traceback__)\n           STORE_FAST               1 (tb)\n           POP_EXCEPT\n           LOAD_CONST               0 (None)\n           STORE_FAST               0 (e)\n           DELETE_FAST              0 (e)\n\n%3d        LOAD_FAST                1 (tb)\n           RETURN_VALUE\n        >> LOAD_CONST               0 (None)\n           STORE_FAST               0 (e)\n           DELETE_FAST              0 (e)\n           RERAISE                  1\n\n%3d     >> RERAISE                  0\n        >> COPY                     3\n           POP_EXCEPT\n           RERAISE                  1\nExceptionTable:\n4 rows\n' % (TRACEBACK_CODE.co_firstlineno, TRACEBACK_CODE.co_firstlineno + 1, TRACEBACK_CODE.co_firstlineno + 2, TRACEBACK_CODE.co_firstlineno + 5, TRACEBACK_CODE.co_firstlineno + 3, TRACEBACK_CODE.co_firstlineno + 4, TRACEBACK_CODE.co_firstlineno + 5, TRACEBACK_CODE.co_firstlineno + 3)

def _fstring(a, b, c, d):
    return f'{a} {b:4} {c!r} {d!r:4}'

dis_fstring = "%3d        RESUME                   0\n\n%3d        LOAD_FAST                0 (a)\n           FORMAT_VALUE             0\n           LOAD_CONST               1 (' ')\n           LOAD_FAST                1 (b)\n           LOAD_CONST               2 ('4')\n           FORMAT_VALUE             4 (with format)\n           LOAD_CONST               1 (' ')\n           LOAD_FAST                2 (c)\n           FORMAT_VALUE             2 (repr)\n           LOAD_CONST               1 (' ')\n           LOAD_FAST                3 (d)\n           LOAD_CONST               2 ('4')\n           FORMAT_VALUE             6 (repr, with format)\n           BUILD_STRING             7\n           RETURN_VALUE\n" % (_fstring.__code__.co_firstlineno, _fstring.__code__.co_firstlineno + 1)

def _with(c):
    with c:
        x = 1
    y = 2

dis_with = '%3d        RESUME                   0\n\n%3d        LOAD_FAST                0 (c)\n           BEFORE_WITH\n           POP_TOP\n\n%3d        LOAD_CONST               1 (1)\n           STORE_FAST               1 (x)\n\n%3d        LOAD_CONST               0 (None)\n           LOAD_CONST               0 (None)\n           LOAD_CONST               0 (None)\n           CALL                     2\n           POP_TOP\n\n%3d        LOAD_CONST               2 (2)\n           STORE_FAST               2 (y)\n           RETURN_CONST             0 (None)\n\n%3d     >> PUSH_EXC_INFO\n           WITH_EXCEPT_START\n           POP_JUMP_IF_TRUE         1 (to 42)\n           RERAISE                  2\n        >> POP_TOP\n           POP_EXCEPT\n           POP_TOP\n           POP_TOP\n\n%3d        LOAD_CONST               2 (2)\n           STORE_FAST               2 (y)\n           RETURN_CONST             0 (None)\n        >> COPY                     3\n           POP_EXCEPT\n           RERAISE                  1\nExceptionTable:\n2 rows\n' % (_with.__code__.co_firstlineno, _with.__code__.co_firstlineno + 1, _with.__code__.co_firstlineno + 2, _with.__code__.co_firstlineno + 1, _with.__code__.co_firstlineno + 3, _with.__code__.co_firstlineno + 1, _with.__code__.co_firstlineno + 3)

async def _asyncwith(c):
    async with c:
        x = 1
    y = 2

dis_asyncwith = '%3d        RETURN_GENERATOR\n           POP_TOP\n           RESUME                   0\n\n%3d        LOAD_FAST                0 (c)\n           BEFORE_ASYNC_WITH\n           GET_AWAITABLE            1\n           LOAD_CONST               0 (None)\n        >> SEND                     3 (to 24)\n           YIELD_VALUE              2\n           RESUME                   3\n           JUMP_BACKWARD_NO_INTERRUPT     5 (to 14)\n        >> END_SEND\n           POP_TOP\n\n%3d        LOAD_CONST               1 (1)\n           STORE_FAST               1 (x)\n\n%3d        LOAD_CONST               0 (None)\n           LOAD_CONST               0 (None)\n           LOAD_CONST               0 (None)\n           CALL                     2\n           GET_AWAITABLE            2\n           LOAD_CONST               0 (None)\n        >> SEND                     3 (to 60)\n           YIELD_VALUE              2\n           RESUME                   3\n           JUMP_BACKWARD_NO_INTERRUPT     5 (to 50)\n        >> END_SEND\n           POP_TOP\n\n%3d        LOAD_CONST               2 (2)\n           STORE_FAST               2 (y)\n           RETURN_CONST             0 (None)\n\n%3d     >> CLEANUP_THROW\n           JUMP_BACKWARD           25 (to 24)\n        >> CLEANUP_THROW\n           JUMP_BACKWARD            9 (to 60)\n        >> PUSH_EXC_INFO\n           WITH_EXCEPT_START\n           GET_AWAITABLE            2\n           LOAD_CONST               0 (None)\n        >> SEND                     4 (to 98)\n           YIELD_VALUE              3\n           RESUME                   3\n           JUMP_BACKWARD_NO_INTERRUPT     5 (to 86)\n        >> CLEANUP_THROW\n        >> END_SEND\n           POP_JUMP_IF_TRUE         1 (to 104)\n           RERAISE                  2\n        >> POP_TOP\n           POP_EXCEPT\n           POP_TOP\n           POP_TOP\n\n%3d        LOAD_CONST               2 (2)\n           STORE_FAST               2 (y)\n           RETURN_CONST             0 (None)\n        >> COPY                     3\n           POP_EXCEPT\n           RERAISE                  1\n        >> CALL_INTRINSIC_1         3 (INTRINSIC_STOPITERATION_ERROR)\n           RERAISE                  1\nExceptionTable:\n12 rows\n' % (_asyncwith.__code__.co_firstlineno, _asyncwith.__code__.co_firstlineno + 1, _asyncwith.__code__.co_firstlineno + 2, _asyncwith.__code__.co_firstlineno + 1, _asyncwith.__code__.co_firstlineno + 3, _asyncwith.__code__.co_firstlineno + 1, _asyncwith.__code__.co_firstlineno + 3)

def _tryfinally(a, b):
    try:
        return a
    finally:
        b()

def _tryfinallyconst(b):
    try:
        return 1
    finally:
        b()

dis_tryfinally = '%3d        RESUME                   0\n\n%3d        NOP\n\n%3d        LOAD_FAST                0 (a)\n\n%3d        PUSH_NULL\n           LOAD_FAST                1 (b)\n           CALL                     0\n           POP_TOP\n           RETURN_VALUE\n        >> PUSH_EXC_INFO\n           PUSH_NULL\n           LOAD_FAST                1 (b)\n           CALL                     0\n           POP_TOP\n           RERAISE                  0\n        >> COPY                     3\n           POP_EXCEPT\n           RERAISE                  1\nExceptionTable:\n2 rows\n' % (_tryfinally.__code__.co_firstlineno, _tryfinally.__code__.co_firstlineno + 1, _tryfinally.__code__.co_firstlineno + 2, _tryfinally.__code__.co_firstlineno + 4)

dis_tryfinallyconst = '%3d        RESUME                   0\n\n%3d        NOP\n\n%3d        NOP\n\n%3d        PUSH_NULL\n           LOAD_FAST                0 (b)\n           CALL                     0\n           POP_TOP\n           RETURN_CONST             1 (1)\n           PUSH_EXC_INFO\n           PUSH_NULL\n           LOAD_FAST                0 (b)\n           CALL                     0\n           POP_TOP\n           RERAISE                  0\n        >> COPY                     3\n           POP_EXCEPT\n           RERAISE                  1\nExceptionTable:\n1 row\n' % (_tryfinallyconst.__code__.co_firstlineno, _tryfinallyconst.__code__.co_firstlineno + 1, _tryfinallyconst.__code__.co_firstlineno + 2, _tryfinallyconst.__code__.co_firstlineno + 4)

def _g(x):
    yield x

async def _ag(x):
    yield x

async def _co(x):
    async for item in _ag(x):
        pass

def _h(y):

    def foo(x):
        """funcdoc"""
        return list((x + z for z in y))
    return foo

dis_nested_0 = '           MAKE_CELL                0 (y)\n\n%3d        RESUME                   0\n\n%3d        LOAD_CLOSURE             0 (y)\n           BUILD_TUPLE              1\n           LOAD_CONST               1 (<code object foo at 0x..., file "%s", line %d>)\n           MAKE_FUNCTION            8 (closure)\n           STORE_FAST               1 (foo)\n\n%3d        LOAD_FAST                1 (foo)\n           RETURN_VALUE\n' % (_h.__code__.co_firstlineno, _h.__code__.co_firstlineno + 1, __file__, _h.__code__.co_firstlineno + 1, _h.__code__.co_firstlineno + 4)

dis_nested_1 = '%s\nDisassembly of <code object foo at 0x..., file "%s", line %d>:\n           COPY_FREE_VARS           1\n           MAKE_CELL                0 (x)\n\n%3d        RESUME                   0\n\n%3d        LOAD_GLOBAL              1 (NULL + list)\n           LOAD_CLOSURE             0 (x)\n           BUILD_TUPLE              1\n           LOAD_CONST               1 (<code object <genexpr> at 0x..., file "%s", line %d>)\n           MAKE_FUNCTION            8 (closure)\n           LOAD_DEREF               1 (y)\n           GET_ITER\n           CALL                     0\n           CALL                     1\n           RETURN_VALUE\n' % (dis_nested_0, __file__, _h.__code__.co_firstlineno + 1, _h.__code__.co_firstlineno + 1, _h.__code__.co_firstlineno + 3, __file__, _h.__code__.co_firstlineno + 3)

dis_nested_2 = '%s\nDisassembly of <code object <genexpr> at 0x..., file "%s", line %d>:\n           COPY_FREE_VARS           1\n\n%3d        RETURN_GENERATOR\n           POP_TOP\n           RESUME                   0\n           LOAD_FAST                0 (.0)\n        >> FOR_ITER                 9 (to 32)\n           STORE_FAST               1 (z)\n           LOAD_DEREF               2 (x)\n           LOAD_FAST                1 (z)\n           BINARY_OP                0 (+)\n           YIELD_VALUE              1\n           RESUME                   1\n           POP_TOP\n           JUMP_BACKWARD           11 (to 10)\n        >> END_FOR\n           RETURN_CONST             0 (None)\n        >> CALL_INTRINSIC_1         3 (INTRINSIC_STOPITERATION_ERROR)\n           RERAISE                  1\nExceptionTable:\n1 row\n' % (dis_nested_1, __file__, _h.__code__.co_firstlineno + 3, _h.__code__.co_firstlineno + 3)

def load_test(x, y=0):
    a, b = (x, y)
    return (a, b)

dis_load_test_quickened_code = '%3d           0 RESUME                   0\n\n%3d           2 LOAD_FAST__LOAD_FAST     0 (x)\n              4 LOAD_FAST                1 (y)\n              6 STORE_FAST__STORE_FAST     3 (b)\n              8 STORE_FAST__LOAD_FAST     2 (a)\n\n%3d          10 LOAD_FAST__LOAD_FAST     2 (a)\n             12 LOAD_FAST                3 (b)\n             14 BUILD_TUPLE              2\n             16 RETURN_VALUE\n' % (load_test.__code__.co_firstlineno, load_test.__code__.co_firstlineno + 1, load_test.__code__.co_firstlineno + 2)

def loop_test():
    for i in [1, 2, 3] * 3:
        load_test(i)

dis_loop_test_quickened_code = '%3d        RESUME                   0\n\n%3d        BUILD_LIST               0\n           LOAD_CONST               1 ((1, 2, 3))\n           LIST_EXTEND              1\n           LOAD_CONST               2 (3)\n           BINARY_OP                5 (*)\n           GET_ITER\n        >> FOR_ITER_LIST           13 (to 46)\n           STORE_FAST               0 (i)\n\n%3d        LOAD_GLOBAL_MODULE       1 (NULL + load_test)\n           LOAD_FAST                0 (i)\n           CALL_PY_WITH_DEFAULTS     1\n           POP_TOP\n           JUMP_BACKWARD           15 (to 16)\n\n%3d     >> END_FOR\n           RETURN_CONST             0 (None)\n' % (loop_test.__code__.co_firstlineno, loop_test.__code__.co_firstlineno + 1, loop_test.__code__.co_firstlineno + 2, loop_test.__code__.co_firstlineno + 1)

def extended_arg_quick():
    *_, _ = ...

dis_extended_arg_quick_code = '%3d           0 RESUME                   0\n\n%3d           2 LOAD_CONST               1 (Ellipsis)\n              4 EXTENDED_ARG             1\n              6 UNPACK_EX              256\n              8 STORE_FAST               0 (_)\n             10 STORE_FAST               0 (_)\n             12 RETURN_CONST             0 (None)\n' % (extended_arg_quick.__code__.co_firstlineno, extended_arg_quick.__code__.co_firstlineno + 1)

ADAPTIVE_WARMUP_DELAY = 2

if dis.code_info.__doc__ is None:
    code_info_consts = '0: None'
else:
    code_info_consts = "0: 'Formatted details of methods, functions, or code.'"

code_info_code_info = f'Name:              code_info\nFilename:          (.*)\nArgument count:    1\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  1\nStack size:        \\d+\nFlags:             OPTIMIZED, NEWLOCALS\nConstants:\n   {code_info_consts}\nNames:\n   0: _format_code_info\n   1: _get_code_object\nVariable names:\n   0: x'

@staticmethod
def tricky(a, b, /, x, y, z=True, *args, c, d, e=[], **kwds):

    def f(c=c):
        print(a, b, x, y, z, c, d, e, f)
    yield (a, b, x, y, z, c, d, e, f)

code_info_tricky = 'Name:              tricky\nFilename:          (.*)\nArgument count:    5\nPositional-only arguments: 2\nKw-only arguments: 3\nNumber of locals:  10\nStack size:        \\d+\nFlags:             OPTIMIZED, NEWLOCALS, VARARGS, VARKEYWORDS, GENERATOR\nConstants:\n   0: None\n   1: <code object f at (.*), file "(.*)", line (.*)>\nVariable names:\n   0: a\n   1: b\n   2: x\n   3: y\n   4: z\n   5: c\n   6: d\n   7: e\n   8: args\n   9: kwds\nCell variables:\n   0: [abedfxyz]\n   1: [abedfxyz]\n   2: [abedfxyz]\n   3: [abedfxyz]\n   4: [abedfxyz]\n   5: [abedfxyz]'

co_tricky_nested_f = tricky.__func__.__code__.co_consts[1]

code_info_tricky_nested_f = 'Filename:          (.*)\nArgument count:    1\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  1\nStack size:        \\d+\nFlags:             OPTIMIZED, NEWLOCALS, NESTED\nConstants:\n   0: None\nNames:\n   0: print\nVariable names:\n   0: c\nFree variables:\n   0: [abedfxyz]\n   1: [abedfxyz]\n   2: [abedfxyz]\n   3: [abedfxyz]\n   4: [abedfxyz]\n   5: [abedfxyz]'

code_info_expr_str = 'Name:              <module>\nFilename:          <disassembly>\nArgument count:    0\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  0\nStack size:        \\d+\nFlags:             0x0\nConstants:\n   0: 1\nNames:\n   0: x'

code_info_simple_stmt_str = 'Name:              <module>\nFilename:          <disassembly>\nArgument count:    0\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  0\nStack size:        \\d+\nFlags:             0x0\nConstants:\n   0: 1\n   1: None\nNames:\n   0: x'

code_info_compound_stmt_str = 'Name:              <module>\nFilename:          <disassembly>\nArgument count:    0\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  0\nStack size:        \\d+\nFlags:             0x0\nConstants:\n   0: 0\n   1: 1\nNames:\n   0: x'

async def async_def():
    await 1
    async for a in b:
        pass
    async with c as d:
        pass

code_info_async_def = 'Name:              async_def\nFilename:          (.*)\nArgument count:    0\nPositional-only arguments: 0\nKw-only arguments: 0\nNumber of locals:  2\nStack size:        \\d+\nFlags:             OPTIMIZED, NEWLOCALS, COROUTINE\nConstants:\n   0: None\n   1: 1\nNames:\n   0: b\n   1: c\nVariable names:\n   0: a\n   1: d'

def outer(a=1, b=2):

    def f(c=3, d=4):

        def inner(e=5, f=6):
            print(a, b, c, d, e, f)
        print(a, b, c, d)
        return inner
    print(a, b, '', 1, [], {}, 'Hello world!')
    return f

def jumpy():
    for i in range(10):
        print(i)
        if i < 4:
            continue
        if i > 6:
            break
    else:
        print('I can haz else clause?')
    while i:
        print(i)
        i -= 1
        if i > 6:
            continue
        if i < 4:
            break
    else:
        print('Who let lolcatz into this test suite?')
    try:
        1 / 0
    except ZeroDivisionError:
        print('Here we go, here we go, here we go...')
    else:
        with i as dodgy:
            print('Never reach this')
    finally:
        print("OK, now we're done")

expected_outer_line = 1

_line_offset = outer.__code__.co_firstlineno - 1

code_object_f = outer.__code__.co_consts[1]

expected_f_line = code_object_f.co_firstlineno - _line_offset

code_object_inner = code_object_f.co_consts[1]

expected_inner_line = code_object_inner.co_firstlineno - _line_offset

expected_jumpy_line = 1

def _stringify_instruction(instr):
    return str(instr._replace(positions=None))

def _prepare_test_cases():
    _instructions = dis.get_instructions(outer, first_line=expected_outer_line)
    print('expected_opinfo_outer = [\n  ', ',\n  '.join(map(_stringify_instruction, _instructions)), ',\n]', sep='')
    _instructions = dis.get_instructions(outer(), first_line=expected_f_line)
    print('expected_opinfo_f = [\n  ', ',\n  '.join(map(_stringify_instruction, _instructions)), ',\n]', sep='')
    _instructions = dis.get_instructions(outer()(), first_line=expected_inner_line)
    print('expected_opinfo_inner = [\n  ', ',\n  '.join(map(_stringify_instruction, _instructions)), ',\n]', sep='')
    _instructions = dis.get_instructions(jumpy, first_line=expected_jumpy_line)
    print('expected_opinfo_jumpy = [\n  ', ',\n  '.join(map(_stringify_instruction, _instructions)), ',\n]', sep='')
    dis.dis(outer)

Instruction = dis.Instruction

expected_opinfo_outer = [Instruction(opname='MAKE_CELL', opcode=135, arg=0, argval='a', argrepr='a', offset=0, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='MAKE_CELL', opcode=135, arg=1, argval='b', argrepr='b', offset=2, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RESUME', opcode=151, arg=0, argval=0, argrepr='', offset=4, starts_line=1, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=5, argval=(3, 4), argrepr='(3, 4)', offset=6, starts_line=2, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=0, argval='a', argrepr='a', offset=8, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=1, argval='b', argrepr='b', offset=10, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BUILD_TUPLE', opcode=102, arg=2, argval=2, argrepr='', offset=12, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=1, argval=code_object_f, argrepr=repr(code_object_f), offset=14, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='MAKE_FUNCTION', opcode=132, arg=9, argval=9, argrepr='defaults, closure', offset=16, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='STORE_FAST', opcode=125, arg=2, argval='f', argrepr='f', offset=18, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=1, argval='print', argrepr='NULL + print', offset=20, starts_line=7, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=0, argval='a', argrepr='a', offset=30, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=1, argval='b', argrepr='b', offset=32, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=2, argval='', argrepr="''", offset=34, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=3, argval=1, argrepr='1', offset=36, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BUILD_LIST', opcode=103, arg=0, argval=0, argrepr='', offset=38, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BUILD_MAP', opcode=105, arg=0, argval=0, argrepr='', offset=40, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=4, argval='Hello world!', argrepr="'Hello world!'", offset=42, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=7, argval=7, argrepr='', offset=44, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=52, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=2, argval='f', argrepr='f', offset=54, starts_line=8, is_jump_target=False, positions=None), Instruction(opname='RETURN_VALUE', opcode=83, arg=None, argval=None, argrepr='', offset=56, starts_line=None, is_jump_target=False, positions=None)]

expected_opinfo_f = [Instruction(opname='COPY_FREE_VARS', opcode=149, arg=2, argval=2, argrepr='', offset=0, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='MAKE_CELL', opcode=135, arg=0, argval='c', argrepr='c', offset=2, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='MAKE_CELL', opcode=135, arg=1, argval='d', argrepr='d', offset=4, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RESUME', opcode=151, arg=0, argval=0, argrepr='', offset=6, starts_line=2, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=2, argval=(5, 6), argrepr='(5, 6)', offset=8, starts_line=3, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=3, argval='a', argrepr='a', offset=10, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=4, argval='b', argrepr='b', offset=12, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=0, argval='c', argrepr='c', offset=14, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CLOSURE', opcode=136, arg=1, argval='d', argrepr='d', offset=16, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BUILD_TUPLE', opcode=102, arg=4, argval=4, argrepr='', offset=18, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=1, argval=code_object_inner, argrepr=repr(code_object_inner), offset=20, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='MAKE_FUNCTION', opcode=132, arg=9, argval=9, argrepr='defaults, closure', offset=22, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='STORE_FAST', opcode=125, arg=2, argval='inner', argrepr='inner', offset=24, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=1, argval='print', argrepr='NULL + print', offset=26, starts_line=5, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=3, argval='a', argrepr='a', offset=36, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=4, argval='b', argrepr='b', offset=38, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=0, argval='c', argrepr='c', offset=40, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=1, argval='d', argrepr='d', offset=42, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=4, argval=4, argrepr='', offset=44, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=52, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=2, argval='inner', argrepr='inner', offset=54, starts_line=6, is_jump_target=False, positions=None), Instruction(opname='RETURN_VALUE', opcode=83, arg=None, argval=None, argrepr='', offset=56, starts_line=None, is_jump_target=False, positions=None)]

expected_opinfo_inner = [Instruction(opname='COPY_FREE_VARS', opcode=149, arg=4, argval=4, argrepr='', offset=0, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RESUME', opcode=151, arg=0, argval=0, argrepr='', offset=2, starts_line=3, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=1, argval='print', argrepr='NULL + print', offset=4, starts_line=4, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=2, argval='a', argrepr='a', offset=14, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=3, argval='b', argrepr='b', offset=16, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=4, argval='c', argrepr='c', offset=18, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_DEREF', opcode=137, arg=5, argval='d', argrepr='d', offset=20, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='e', argrepr='e', offset=22, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=1, argval='f', argrepr='f', offset=24, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=6, argval=6, argrepr='', offset=26, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=34, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RETURN_CONST', opcode=121, arg=0, argval=None, argrepr='None', offset=36, starts_line=None, is_jump_target=False, positions=None)]

expected_opinfo_jumpy = [Instruction(opname='RESUME', opcode=151, arg=0, argval=0, argrepr='', offset=0, starts_line=1, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=1, argval='range', argrepr='NULL + range', offset=2, starts_line=3, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=1, argval=10, argrepr='10', offset=12, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=14, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='GET_ITER', opcode=68, arg=None, argval=None, argrepr='', offset=22, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='FOR_ITER', opcode=93, arg=26, argval=80, argrepr='to 80', offset=24, starts_line=None, is_jump_target=True, positions=None), Instruction(opname='STORE_FAST', opcode=125, arg=0, argval='i', argrepr='i', offset=28, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=30, starts_line=4, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=40, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=42, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=50, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=52, starts_line=5, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=2, argval=4, argrepr='4', offset=54, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COMPARE_OP', opcode=107, arg=2, argval='<', argrepr='<', offset=56, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=1, argval=64, argrepr='to 64', offset=60, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=20, argval=24, argrepr='to 24', offset=62, starts_line=6, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=64, starts_line=7, is_jump_target=True, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=3, argval=6, argrepr='6', offset=66, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COMPARE_OP', opcode=107, arg=68, argval='>', argrepr='>', offset=68, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_TRUE', opcode=115, arg=1, argval=76, argrepr='to 76', offset=72, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=26, argval=24, argrepr='to 24', offset=74, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=76, starts_line=8, is_jump_target=True, positions=None), Instruction(opname='JUMP_FORWARD', opcode=110, arg=12, argval=104, argrepr='to 104', offset=78, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='END_FOR', opcode=4, arg=None, argval=None, argrepr='', offset=80, starts_line=3, is_jump_target=True, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=82, starts_line=10, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=4, argval='I can haz else clause?', argrepr="'I can haz else clause?'", offset=92, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=94, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=102, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST_CHECK', opcode=127, arg=0, argval='i', argrepr='i', offset=104, starts_line=11, is_jump_target=True, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=31, argval=170, argrepr='to 170', offset=106, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=108, starts_line=12, is_jump_target=True, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=118, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=120, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=128, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=130, starts_line=13, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=5, argval=1, argrepr='1', offset=132, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BINARY_OP', opcode=122, arg=23, argval=23, argrepr='-=', offset=134, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='STORE_FAST', opcode=125, arg=0, argval='i', argrepr='i', offset=138, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=140, starts_line=14, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=3, argval=6, argrepr='6', offset=142, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COMPARE_OP', opcode=107, arg=68, argval='>', argrepr='>', offset=144, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=1, argval=152, argrepr='to 152', offset=148, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=24, argval=104, argrepr='to 104', offset=150, starts_line=15, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=152, starts_line=16, is_jump_target=True, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=2, argval=4, argrepr='4', offset=154, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COMPARE_OP', opcode=107, arg=2, argval='<', argrepr='<', offset=156, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=1, argval=164, argrepr='to 164', offset=160, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_FORWARD', opcode=110, arg=14, argval=192, argrepr='to 192', offset=162, starts_line=17, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=164, starts_line=11, is_jump_target=True, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=1, argval=170, argrepr='to 170', offset=166, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=31, argval=108, argrepr='to 108', offset=168, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=170, starts_line=19, is_jump_target=True, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=6, argval='Who let lolcatz into this test suite?', argrepr="'Who let lolcatz into this test suite?'", offset=180, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=182, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=190, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='NOP', opcode=9, arg=None, argval=None, argrepr='', offset=192, starts_line=20, is_jump_target=True, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=5, argval=1, argrepr='1', offset=194, starts_line=21, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=7, argval=0, argrepr='0', offset=196, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='BINARY_OP', opcode=122, arg=11, argval=11, argrepr='/', offset=198, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=202, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_FAST', opcode=124, arg=0, argval='i', argrepr='i', offset=204, starts_line=25, is_jump_target=False, positions=None), Instruction(opname='BEFORE_WITH', opcode=53, arg=None, argval=None, argrepr='', offset=206, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='STORE_FAST', opcode=125, arg=1, argval='dodgy', argrepr='dodgy', offset=208, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=210, starts_line=26, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=8, argval='Never reach this', argrepr="'Never reach this'", offset=220, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=222, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=230, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=0, argval=None, argrepr='None', offset=232, starts_line=25, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=0, argval=None, argrepr='None', offset=234, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=0, argval=None, argrepr='None', offset=236, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=2, argval=2, argrepr='', offset=238, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=246, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=248, starts_line=28, is_jump_target=True, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=10, argval="OK, now we're done", argrepr='"OK, now we\'re done"', offset=258, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=260, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=268, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RETURN_CONST', opcode=121, arg=0, argval=None, argrepr='None', offset=270, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='PUSH_EXC_INFO', opcode=35, arg=None, argval=None, argrepr='', offset=272, starts_line=25, is_jump_target=False, positions=None), Instruction(opname='WITH_EXCEPT_START', opcode=49, arg=None, argval=None, argrepr='', offset=274, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_TRUE', opcode=115, arg=1, argval=280, argrepr='to 280', offset=276, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=2, argval=2, argrepr='', offset=278, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=280, starts_line=None, is_jump_target=True, positions=None), Instruction(opname='POP_EXCEPT', opcode=89, arg=None, argval=None, argrepr='', offset=282, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=284, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=286, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=21, argval=248, argrepr='to 248', offset=288, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COPY', opcode=120, arg=3, argval=3, argrepr='', offset=290, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_EXCEPT', opcode=89, arg=None, argval=None, argrepr='', offset=292, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=1, argval=1, argrepr='', offset=294, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='PUSH_EXC_INFO', opcode=35, arg=None, argval=None, argrepr='', offset=296, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=4, argval='ZeroDivisionError', argrepr='ZeroDivisionError', offset=298, starts_line=22, is_jump_target=False, positions=None), Instruction(opname='CHECK_EXC_MATCH', opcode=36, arg=None, argval=None, argrepr='', offset=308, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_JUMP_IF_FALSE', opcode=114, arg=14, argval=340, argrepr='to 340', offset=310, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=312, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=314, starts_line=23, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=9, argval='Here we go, here we go, here we go...', argrepr="'Here we go, here we go, here we go...'", offset=324, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=326, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=334, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_EXCEPT', opcode=89, arg=None, argval=None, argrepr='', offset=336, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='JUMP_BACKWARD', opcode=140, arg=46, argval=248, argrepr='to 248', offset=338, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=0, argval=0, argrepr='', offset=340, starts_line=22, is_jump_target=True, positions=None), Instruction(opname='COPY', opcode=120, arg=3, argval=3, argrepr='', offset=342, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_EXCEPT', opcode=89, arg=None, argval=None, argrepr='', offset=344, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=1, argval=1, argrepr='', offset=346, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='PUSH_EXC_INFO', opcode=35, arg=None, argval=None, argrepr='', offset=348, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='LOAD_GLOBAL', opcode=116, arg=3, argval='print', argrepr='NULL + print', offset=350, starts_line=28, is_jump_target=False, positions=None), Instruction(opname='LOAD_CONST', opcode=100, arg=10, argval="OK, now we're done", argrepr='"OK, now we\'re done"', offset=360, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='CALL', opcode=171, arg=1, argval=1, argrepr='', offset=362, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_TOP', opcode=1, arg=None, argval=None, argrepr='', offset=370, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=0, argval=0, argrepr='', offset=372, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='COPY', opcode=120, arg=3, argval=3, argrepr='', offset=374, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='POP_EXCEPT', opcode=89, arg=None, argval=None, argrepr='', offset=376, starts_line=None, is_jump_target=False, positions=None), Instruction(opname='RERAISE', opcode=119, arg=1, argval=1, argrepr='', offset=378, starts_line=None, is_jump_target=False, positions=None)]

def simple():
    pass

expected_opinfo_simple = [Instruction(opname='RESUME', opcode=151, arg=0, argval=0, argrepr='', offset=0, starts_line=simple.__code__.co_firstlineno, is_jump_target=False, positions=None), Instruction(opname='RETURN_CONST', opcode=121, arg=0, argval=None, argrepr='None', offset=2, starts_line=None, is_jump_target=False)]

class InstructionTestCase(BytecodeTestCase):

    def assertInstructionsEqual(self, instrs_1, instrs_2, /):
        instrs_1 = [instr_1._replace(positions=None) for instr_1 in instrs_1]
        instrs_2 = [instr_2._replace(positions=None) for instr_2 in instrs_2]
        self.assertEqual(instrs_1, instrs_2)


# --- test body ---
def assert_exception_table_increasing(lines):
    prev_start, prev_end = (-1, -1)
    count = 0
    for line in lines:
        m = re.match('  (\\d+) to (\\d+) -> \\d+ \\[\\d+\\]', line)
        start, end = [int(g) for g in m.groups()]

        assert end >= start

        assert start > prev_end
        prev_start, prev_end = (start, end)
        count += 1
    return count

def assert_offsets_increasing(text, delta):
    expected_offset = 0
    lines = text.splitlines()
    start, end = find_offset_column(lines)
    for line in lines:
        if not line:
            continue
        if line.startswith('Disassembly'):
            expected_offset = 0
            continue
        if line.startswith('Exception'):
            break
        offset = int(line[start:end])

        assert offset >= expected_offset
        expected_offset = offset + delta

def do_disassembly_compare(got, expected, with_offsets=False):
    if not with_offsets:
        assert_offsets_increasing(got, 2)
        got = strip_offsets(got)
    if got != expected:
        got = strip_addresses(got)

    assert got == expected

def find_offset_column(lines):
    for line in lines:
        if line and (not line.startswith('Disassembly')):
            break
    else:
        return (0, 0)
    offset = 5
    while line[offset] == ' ':
        offset += 1
    if line[offset] == '>':
        offset += 2
    while line[offset] == ' ':
        offset += 1
    end = offset
    while line[end] in '0123456789':
        end += 1
    return (end - 5, end)

def strip_addresses(text):
    return re.sub('\\b0x[0-9A-Fa-f]+\\b', '0x...', text)

def strip_offsets(text):
    lines = text.splitlines(True)
    start, end = find_offset_column(lines)
    res = []
    lines = iter(lines)
    for line in lines:
        if line.startswith('Exception'):
            res.append(line)
            break
        if not line or line.startswith('Disassembly'):
            res.append(line)
        else:
            res.append(line[:start] + line[end:])
    num_rows = assert_exception_table_increasing(lines)
    if num_rows:
        res.append(f"{num_rows} row{('s' if num_rows > 1 else '')}\n")
    return ''.join(res)
for obj in [_f, _C(1).__init__, 'a=1', _f.__code__]:
    b = dis.Bytecode(obj)

    assert isinstance(b.codeobj, types.CodeType)

try:
    dis.Bytecode(object())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BytecodeTests::test_instantiation: ok")
