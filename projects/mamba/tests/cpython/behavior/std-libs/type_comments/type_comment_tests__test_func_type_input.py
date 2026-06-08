# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_comments"
# dimension = "behavior"
# case = "type_comment_tests__test_func_type_input"
# subject = "cpython.test_type_comments.TypeCommentTests.test_func_type_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_comments.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_comments.py::TypeCommentTests::test_func_type_input
"""Auto-ported test: TypeCommentTests::test_func_type_input (CPython 3.12 oracle)."""


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

def parse_func_type_input(source):
    return ast.parse(source, '<unknown>', 'func_type')
tree = parse_func_type_input('() -> int')

assert tree.argtypes == []

assert tree.returns.id == 'int'
tree = parse_func_type_input('(int) -> List[str]')

assert len(tree.argtypes) == 1
arg = tree.argtypes[0]

assert arg.id == 'int'

assert tree.returns.value.id == 'List'

assert tree.returns.slice.id == 'str'
tree = parse_func_type_input('(int, *str, **Any) -> float')

assert tree.argtypes[0].id == 'int'

assert tree.argtypes[1].id == 'str'

assert tree.argtypes[2].id == 'Any'

assert tree.returns.id == 'float'
tree = parse_func_type_input('(*int) -> None')

assert tree.argtypes[0].id == 'int'
tree = parse_func_type_input('(**int) -> None')

assert tree.argtypes[0].id == 'int'
tree = parse_func_type_input('(*int, **str) -> None')

assert tree.argtypes[0].id == 'int'

assert tree.argtypes[1].id == 'str'
try:
    tree = parse_func_type_input('(int, *str, *Any) -> float')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
try:
    tree = parse_func_type_input('(int, **str, Any) -> float')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
try:
    tree = parse_func_type_input('(**int, **str) -> float')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TypeCommentTests::test_func_type_input: ok")
