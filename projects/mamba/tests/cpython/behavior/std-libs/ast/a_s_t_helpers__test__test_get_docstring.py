# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_get_docstring"
# subject = "cpython.test_ast.ASTHelpers_Test.test_get_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


node = ast.parse('"""line one\n  line two"""')
assert ast.get_docstring(node) == 'line one\nline two'
node = ast.parse('class foo:\n  """line one\n  line two"""')
assert ast.get_docstring(node.body[0]) == 'line one\nline two'
node = ast.parse('def foo():\n  """line one\n  line two"""')
assert ast.get_docstring(node.body[0]) == 'line one\nline two'
node = ast.parse('async def foo():\n  """spam\n  ham"""')
assert ast.get_docstring(node.body[0]) == 'spam\nham'
node = ast.parse('async def foo():\n  """spam\n  ham"""')
assert ast.get_docstring(node.body[0], clean=False) == 'spam\n  ham'
node = ast.parse('x')
try:
    ast.get_docstring(node.body[0])
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("ASTHelpers_Test::test_get_docstring: ok")
