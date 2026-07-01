# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_call"
# subject = "cpython.test_ast.EndPositionTests.test_call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'func(x, y=2, **kw)'
call = _parse_value(s)
_check_content(s, call.func, 'func')
_check_content(s, call.keywords[0].value, '2')
_check_content(s, call.keywords[1].value, 'kw')

print("EndPositionTests::test_call: ok")
