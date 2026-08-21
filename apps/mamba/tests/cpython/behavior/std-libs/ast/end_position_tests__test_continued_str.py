# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_continued_str"
# subject = "cpython.test_ast.EndPositionTests.test_continued_str"
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
s = dedent('\n            x = "first part" \\\n            "second part"\n        ').strip()
assign = ast.parse(s).body[0]
_check_end_pos(assign, 2, 13)
_check_end_pos(assign.value, 2, 13)

print("EndPositionTests::test_continued_str: ok")
