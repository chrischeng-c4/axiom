# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "annotations_future_test_case__test_annotations"
# subject = "cpython.test.test_future_stmt.test_future.AnnotationsFutureTestCase.test_annotations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future.py::AnnotationsFutureTestCase::test_annotations
"""Auto-ported test: AnnotationsFutureTestCase::test_annotations (CPython 3.12 oracle)."""


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
eq = assertAnnotationEqual
eq('...')
eq("'some_string'")
eq("u'some_string'")
eq("b'\\xa3'")
eq('Name')
eq('None')
eq('True')
eq('False')
eq('1')
eq('1.0')
eq('1j')
eq('True or False')
eq('True or False or None')
eq('True and False')
eq('True and False and None')
eq('Name1 and Name2 or Name3')
eq('Name1 and (Name2 or Name3)')
eq('Name1 or Name2 and Name3')
eq('(Name1 or Name2) and Name3')
eq('Name1 and Name2 or Name3 and Name4')
eq('Name1 or Name2 and Name3 or Name4')
eq('a + b + (c + d)')
eq('a * b * (c * d)')
eq('(a ** b) ** c ** d')
eq('v1 << 2')
eq('1 >> v2')
eq('1 % finished')
eq('1 + v2 - v3 * 4 ^ 5 ** v6 / 7 // 8')
eq('not great')
eq('not not great')
eq('~great')
eq('+value')
eq('++value')
eq('-1')
eq('~int and not v1 ^ 123 + v2 | True')
eq('a + (not b)')
eq('lambda: None')
eq('lambda arg: None')
eq('lambda a=True: a')
eq('lambda a, b, c=True: a')
eq("lambda a, b, c=True, *, d=1 << v2, e='str': a")
eq("lambda a, b, c=True, *vararg, d, e='str', **kwargs: a + b")
eq("lambda a, /, b, c=True, *vararg, d, e='str', **kwargs: a + b")
eq('lambda x, /: x')
eq('lambda x=1, /: x')
eq('lambda x, /, y: x + y')
eq('lambda x=1, /, y=2: x + y')
eq('lambda x, /, y=1: x + y')
eq('lambda x, /, y=1, *, z=3: x + y + z')
eq('lambda x=1, /, y=2, *, z=3: x + y + z')
eq('lambda x=1, /, y=2, *, z: x + y + z')
eq('lambda x=1, y=2, z=3, /, w=4, *, l, l2: x + y + z + w + l + l2')
eq('lambda x=1, y=2, z=3, /, w=4, *, l, l2, **kwargs: x + y + z + w + l + l2')
eq('lambda x, /, y=1, *, z: x + y + z')
eq('lambda x: lambda y: x + y')
eq('1 if True else 2')
eq('str or None if int or True else str or bytes or None')
eq('str or None if (1 if True else 2) else str or bytes or None')
eq('0 if not x else 1 if x > 0 else -1')
eq('(1 if x > 0 else -1) if x else 0')
eq("{'2.7': dead, '3.7': long_live or die_hard}")
eq("{'2.7': dead, '3.7': long_live or die_hard, **{'3.6': verygood}}")
eq('{**a, **b, **c}')
eq("{'2.7', '3.6', '3.7', '3.8', '3.9', '4.0' if gilectomy else '3.10'}")
eq('{*a, *b, *c}')
eq("({'a': 'b'}, True or False, +value, 'string', b'bytes') or None")
eq('()')
eq('(a,)')
eq('(a, b)')
eq('(a, b, c)')
eq('(*a, *b, *c)')
eq('[]')
eq('[1, 2, 3, 4, 5, 6, 7, 8, 9, 10 or A, 11 or B, 12 or C]')
eq('[*a, *b, *c]')
eq('{i for i in (1, 2, 3)}')
eq('{i ** 2 for i in (1, 2, 3)}')
eq("{i ** 2 for i, _ in ((1, 'a'), (2, 'b'), (3, 'c'))}")
eq('{i ** 2 + j for i in (1, 2, 3) for j in (1, 2, 3)}')
eq('[i for i in (1, 2, 3)]')
eq('[i ** 2 for i in (1, 2, 3)]')
eq("[i ** 2 for i, _ in ((1, 'a'), (2, 'b'), (3, 'c'))]")
eq('[i ** 2 + j for i in (1, 2, 3) for j in (1, 2, 3)]')
eq('(i for i in (1, 2, 3))')
eq('(i ** 2 for i in (1, 2, 3))')
eq("(i ** 2 for i, _ in ((1, 'a'), (2, 'b'), (3, 'c')))")
eq('(i ** 2 + j for i in (1, 2, 3) for j in (1, 2, 3))')
eq('{i: 0 for i in (1, 2, 3)}')
eq("{i: j for i, j in ((1, 'a'), (2, 'b'), (3, 'c'))}")
eq('[(x, y) for x, y in (a, b)]')
eq('[(x,) for x, in (a,)]')
eq('Python3 > Python2 > COBOL')
eq('Life is Life')
eq('call()')
eq('call(arg)')
eq("call(kwarg='hey')")
eq("call(arg, kwarg='hey')")
eq("call(arg, *args, another, kwarg='hey')")
eq("call(arg, another, kwarg='hey', **kwargs, kwarg2='ho')")
eq('lukasz.langa.pl')
eq('call.me(maybe)')
eq('1 .real')
eq('1.0.real')
eq('....__class__')
eq('list[str]')
eq('dict[str, int]')
eq('set[str,]')
eq('tuple[()]')
eq('tuple[str, ...]')
eq('tuple[str, *types]')
eq('tuple[str, int, (str, int)]')
eq('tuple[*int, str, str, (str, int)]')
eq('tuple[str, int, float, dict[str, int]]')
eq('slice[0]')
eq('slice[0:1]')
eq('slice[0:1:2]')
eq('slice[:]')
eq('slice[:-1]')
eq('slice[1:]')
eq('slice[::-1]')
eq('slice[:,]')
eq('slice[1:2,]')
eq('slice[1:2:3,]')
eq('slice[1:2, 1]')
eq('slice[1:2, 2, 3]')
eq('slice[()]')
eq('slice[*Ts,]')
eq('slice[1, *Ts]')
eq('slice[*Ts, 2]')
eq('slice[1, *Ts, 2]')
eq('slice[*Ts, *Ts]')
eq('slice[1, *Ts, *Ts]')
eq('slice[*Ts, 1, *Ts]')
eq('slice[*Ts, *Ts, 1]')
eq('slice[1, *Ts, *Ts, 2]')
eq('slice[1:2, *Ts]')
eq('slice[*Ts, 1:2]')
eq('slice[1:2, *Ts, 3:4]')
eq('slice[a, b:c, d:e:f]')
eq('slice[(x for x in a)]')
eq('str or None if sys.version_info[0] > (3,) else str or bytes or None')
eq("f'f-string without formatted values is just a string'")
eq("f'{{NOT a formatted value}}'")
eq("f'some f-string with {a} {few():.2f} {formatted.values!r}'")
eq('f"{f\'{nested} inner\'} outer"')
eq("f'space between opening braces: { {a for a in (1, 2, 3)}}'")
eq("f'{(lambda x: x)}'")
eq("f'{(None if a else lambda x: x)}'")
eq("f'{x}'")
eq("f'{x!r}'")
eq("f'{x!a}'")
eq('[x for x in (a if b else c)]')
eq('[x for x in a if (b if c else d)]')
eq('f(x for x in a)')
eq('f(1, (x for x in a))')
eq('f((x for x in a), 2)')
eq('(((a)))', 'a')
eq('(((a, b)))', '(a, b)')
eq('1 + 2 + 3')
print("AnnotationsFutureTestCase::test_annotations: ok")
