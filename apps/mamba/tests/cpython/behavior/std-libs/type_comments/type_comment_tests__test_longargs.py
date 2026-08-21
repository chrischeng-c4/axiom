# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_comments"
# dimension = "behavior"
# case = "type_comment_tests__test_longargs"
# subject = "cpython.test_type_comments.TypeCommentTests.test_longargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_comments.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_comments.py::TypeCommentTests::test_longargs
"""Auto-ported test: TypeCommentTests::test_longargs (CPython 3.12 oracle)."""


import ast
import sys
import unittest


funcdef = 'def foo():\n    # type: () -> int\n    pass\n\ndef bar():  # type: () -> None\n    pass\n'

asyncdef = 'async def foo():\n    # type: () -> int\n    return await bar()\n\nasync def bar():  # type: () -> int\n    return await bar()\n'

asyncvar = 'async = 12\nawait = 13\n'

asynccomp = 'async def foo(xs):\n    [x async for x in xs]\n'

matmul = 'a = b @ c\n'

fstring = 'a = 42\nf"{a}"\n'

underscorednumber = 'a = 42_42_42\n'

redundantdef = "def foo():  # type: () -> int\n    # type: () -> str\n    return ''\n"

nonasciidef = 'def foo():\n    # type: () -> àçčéñt\n    pass\n'

forstmt = 'for a in []:  # type: int\n    pass\n'

withstmt = 'with context() as a:  # type: int\n    pass\n'

vardecl = 'a = 0  # type: int\n'

ignores = 'def foo():\n    pass  # type: ignore\n\ndef bar():\n    x = 1  # type: ignore\n\ndef baz():\n    pass  # type: ignore[excuse]\n    pass  # type: ignore=excuse\n    pass  # type: ignore [excuse]\n    x = 1  # type: ignore whatever\n'

longargs = 'def fa(\n    a = 1,  # type: A\n):\n    pass\n\ndef fa(\n    a = 1  # type: A\n):\n    pass\n\ndef fa(\n    a = 1,  # type: A\n    /\n):\n    pass\n\ndef fab(\n    a,  # type: A\n    b,  # type: B\n):\n    pass\n\ndef fab(\n    a,  # type: A\n    /,\n    b,  # type: B\n):\n    pass\n\ndef fab(\n    a,  # type: A\n    b   # type: B\n):\n    pass\n\ndef fv(\n    *v,  # type: V\n):\n    pass\n\ndef fv(\n    *v  # type: V\n):\n    pass\n\ndef fk(\n    **k,  # type: K\n):\n    pass\n\ndef fk(\n    **k  # type: K\n):\n    pass\n\ndef fvk(\n    *v,  # type: V\n    **k,  # type: K\n):\n    pass\n\ndef fvk(\n    *v,  # type: V\n    **k  # type: K\n):\n    pass\n\ndef fav(\n    a,  # type: A\n    *v,  # type: V\n):\n    pass\n\ndef fav(\n    a,  # type: A\n    /,\n    *v,  # type: V\n):\n    pass\n\ndef fav(\n    a,  # type: A\n    *v  # type: V\n):\n    pass\n\ndef fak(\n    a,  # type: A\n    **k,  # type: K\n):\n    pass\n\ndef fak(\n    a,  # type: A\n    /,\n    **k,  # type: K\n):\n    pass\n\ndef fak(\n    a,  # type: A\n    **k  # type: K\n):\n    pass\n\ndef favk(\n    a,  # type: A\n    *v,  # type: V\n    **k,  # type: K\n):\n    pass\n\ndef favk(\n    a,  # type: A\n    /,\n    *v,  # type: V\n    **k,  # type: K\n):\n    pass\n\ndef favk(\n    a,  # type: A\n    *v,  # type: V\n    **k  # type: K\n):\n    pass\n'


# --- test body ---
lowest = 4
highest = sys.version_info[1]

def classic_parse(source):
    return ast.parse(source)

def parse(source, feature_version=highest):
    return ast.parse(source, type_comments=True, feature_version=feature_version)

def parse_all(source, minver=lowest, maxver=highest, expected_regex=''):
    for version in range(lowest, highest + 1):
        feature_version = (3, version)
        if minver <= version <= maxver:
            try:
                yield parse(source, feature_version)
            except SyntaxError as err:
                raise SyntaxError(str(err) + f' feature_version={feature_version}')
        else:
            try:
                parse(source, feature_version)
                raise AssertionError('expected SyntaxError')
            except SyntaxError as _aR_e:
                import re as _re_aR
                assert _re_aR.search(expected_regex, str(_aR_e))
for tree in parse_all(longargs, minver=8):
    for t in tree.body:
        todo = set(t.name[1:])

        assert len(t.args.args) + len(t.args.posonlyargs) == len(todo) - bool(t.args.vararg) - bool(t.args.kwarg)

        assert t.name.startswith('f')
        for index, c in enumerate(t.name[1:]):
            todo.remove(c)
            if c == 'v':
                arg = t.args.vararg
            elif c == 'k':
                arg = t.args.kwarg
            else:
                assert 0 <= ord(c) - ord('a') < len(t.args.posonlyargs + t.args.args)
                if index < len(t.args.posonlyargs):
                    arg = t.args.posonlyargs[ord(c) - ord('a')]
                else:
                    arg = t.args.args[ord(c) - ord('a') - len(t.args.posonlyargs)]

            assert arg.arg == c

            assert arg.type_comment == arg.arg.upper()
        assert not todo
tree = classic_parse(longargs)
for t in tree.body:
    for arg in t.args.args + [t.args.vararg, t.args.kwarg]:
        if arg is not None:

            assert arg.type_comment is None
print("TypeCommentTests::test_longargs: ok")
