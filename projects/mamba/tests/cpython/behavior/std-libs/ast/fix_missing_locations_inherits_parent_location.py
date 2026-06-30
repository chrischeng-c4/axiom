# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "fix_missing_locations_inherits_parent_location"
# subject = "ast.fix_missing_locations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
"""ast.fix_missing_locations: children inherit explicit parent locations."""
import ast

leaf = ast.Constant(1)
expr = ast.Expr(leaf)
expr.lineno = 7
expr.col_offset = 2
expr.end_lineno = 9
expr.end_col_offset = 4
module = ast.Module([expr], [])

fixed = ast.fix_missing_locations(module)

assert fixed is module
assert expr.lineno == 7
assert expr.col_offset == 2
assert expr.end_lineno == 9
assert expr.end_col_offset == 4
assert leaf.lineno == 7
assert leaf.col_offset == 2
assert leaf.end_lineno == 9
assert leaf.end_col_offset == 4
print("fix_missing_locations_inherits_parent_location OK")
