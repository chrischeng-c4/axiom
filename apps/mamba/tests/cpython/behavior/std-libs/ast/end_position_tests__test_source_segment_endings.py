# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_endings"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_endings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = 'v = 1\r\nw = 1\nx = 1\n\ry = 1\rz = 1\r\n'
v, w, x, y, z = ast.parse(s).body
_check_content(s, v, 'v = 1')
_check_content(s, w, 'w = 1')
_check_content(s, x, 'x = 1')
_check_content(s, y, 'y = 1')
_check_content(s, z, 'z = 1')

print("EndPositionTests::test_source_segment_endings: ok")
