# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_tuples"
# subject = "cpython.test_ast.EndPositionTests.test_tuples"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s1 = 'x = () ;'
s2 = 'x = 1 , ;'
s3 = 'x = (1 , 2 ) ;'
sm = dedent('\n            x = (\n                a, b,\n            )\n        ').strip()
t1, t2, t3, tm = map(_parse_value, (s1, s2, s3, sm))
_check_content(s1, t1, '()')
_check_content(s2, t2, '1 ,')
_check_content(s3, t3, '(1 , 2 )')
_check_end_pos(tm, 3, 1)

print("EndPositionTests::test_tuples: ok")
