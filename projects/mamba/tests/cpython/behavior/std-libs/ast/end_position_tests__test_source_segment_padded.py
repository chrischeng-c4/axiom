# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_padded"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_padded"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent
s_orig = dedent('\n            class C:\n                def fun(self) -> None:\n                    "ЖЖЖЖЖ"\n        ').strip()
s_method = '    def fun(self) -> None:\n        "ЖЖЖЖЖ"'
cdef = ast.parse(s_orig).body[0]
assert ast.get_source_segment(s_orig, cdef.body[0], padded=True) == s_method

print("EndPositionTests::test_source_segment_padded: ok")
