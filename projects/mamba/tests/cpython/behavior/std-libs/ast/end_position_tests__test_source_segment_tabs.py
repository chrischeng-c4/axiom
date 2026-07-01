# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_tabs"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent
s = dedent('\n            class C:\n              \t\x0c  def fun(self) -> None:\n              \t\x0c      pass\n        ').strip()
s_method = '  \t\x0c  def fun(self) -> None:\n  \t\x0c      pass'
cdef = ast.parse(s).body[0]
assert ast.get_source_segment(s, cdef.body[0], padded=True) == s_method

print("EndPositionTests::test_source_segment_tabs: ok")
