# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_iter_fields"
# subject = "cpython.test_ast.ASTHelpers_Test.test_iter_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


node = ast.parse('foo()', mode='eval')
d = dict(ast.iter_fields(node.body))
assert d.pop('func').id == 'foo'
assert d == {'keywords': [], 'args': []}

print("ASTHelpers_Test::test_iter_fields: ok")
