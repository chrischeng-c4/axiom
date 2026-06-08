# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "annotations_future_test_case__test_infinity_numbers"
# subject = "cpython.test.test_future_stmt.test_future.AnnotationsFutureTestCase.test_infinity_numbers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future.py::AnnotationsFutureTestCase::test_infinity_numbers
"""Auto-ported test: AnnotationsFutureTestCase::test_infinity_numbers (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
template = dedent('\n        from __future__ import annotations\n        def f() -> {ann}:\n            ...\n        def g(arg: {ann}) -> None:\n            ...\n        async def f2() -> {ann}:\n            ...\n        async def g2(arg: {ann}) -> None:\n            ...\n        class H:\n            var: {ann}\n            object.attr: {ann}\n        var: {ann}\n        var2: {ann} = None\n        object.attr: {ann}\n        ')

def _exec_future(code):
    scope = {}
    exec('from __future__ import annotations\n' + code, scope)
    return scope

def assertAnnotationEqual(annotation, expected=None, drop_parens=False, is_tuple=False):
    actual = getActual(annotation)
    if expected is None:
        expected = annotation if not is_tuple else annotation[1:-1]
    if drop_parens:

        assert actual != expected
        actual = actual.replace('(', '').replace(')', '')

    assert actual == expected

def getActual(annotation):
    scope = {}
    exec(template.format(ann=annotation), {}, scope)
    func_ret_ann = scope['f'].__annotations__['return']
    func_arg_ann = scope['g'].__annotations__['arg']
    async_func_ret_ann = scope['f2'].__annotations__['return']
    async_func_arg_ann = scope['g2'].__annotations__['arg']
    var_ann1 = scope['__annotations__']['var']
    var_ann2 = scope['__annotations__']['var2']

    assert func_ret_ann == func_arg_ann

    assert func_ret_ann == async_func_ret_ann

    assert func_ret_ann == async_func_arg_ann

    assert func_ret_ann == var_ann1

    assert func_ret_ann == var_ann2
    return func_ret_ann
inf = '1e' + repr(sys.float_info.max_10_exp + 1)
infj = f'{inf}j'
assertAnnotationEqual('1e1000', expected=inf)
assertAnnotationEqual('1e1000j', expected=infj)
assertAnnotationEqual('-1e1000', expected=f'-{inf}')
assertAnnotationEqual('3+1e1000j', expected=f'3 + {infj}')
assertAnnotationEqual('(1e1000, 1e1000j)', expected=f'({inf}, {infj})')
assertAnnotationEqual("'inf'")
assertAnnotationEqual("('inf', 1e1000, 'infxxx', 1e1000j)", expected=f"('inf', {inf}, 'infxxx', {infj})")
assertAnnotationEqual('(1e1000, (1e1000j,))', expected=f'({inf}, ({infj},))')
print("AnnotationsFutureTestCase::test_infinity_numbers: ok")
