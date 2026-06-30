# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "copy_location_copies_location_attrs"
# subject = "ast.copy_location"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
"""ast.copy_location: copy CPython location attributes without clobbering None starts."""
import ast

old = ast.Constant(1)
old.lineno = 7
old.col_offset = 3
old.end_lineno = None
old.end_col_offset = None

new = ast.Constant(2)
copied = ast.copy_location(new, old)

assert copied is new
assert copied.lineno == 7
assert copied.col_offset == 3
assert copied.end_lineno is None
assert copied.end_col_offset is None
assert copied.value == 2

old_without_start = ast.Constant(1)
old_without_start.lineno = None
old_without_start.col_offset = None
old_without_start.end_lineno = None
old_without_start.end_col_offset = None

preserved_start = ast.Constant(3)
preserved_start.lineno = 11
preserved_start.col_offset = 5
preserved = ast.copy_location(preserved_start, old_without_start)

assert preserved.lineno == 11
assert preserved.col_offset == 5
assert preserved.end_lineno is None
assert preserved.end_col_offset is None
print("copy_location_copies_location_attrs OK")
