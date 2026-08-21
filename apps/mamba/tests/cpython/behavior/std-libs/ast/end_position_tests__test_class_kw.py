# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_class_kw"
# subject = "cpython.test_ast.EndPositionTests.test_class_kw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = 'class S(metaclass=abc.ABCMeta): pass'
cdef = ast.parse(s).body[0]
_check_content(s, cdef.keywords[0].value, 'abc.ABCMeta')

print("EndPositionTests::test_class_kw: ok")
