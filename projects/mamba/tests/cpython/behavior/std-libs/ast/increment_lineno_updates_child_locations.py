# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "increment_lineno_updates_child_locations"
# subject = "ast.increment_lineno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
"""ast.increment_lineno: recursively shifts line attrs and preserves None attrs."""
import ast

leaf = ast.Constant(1)
leaf.lineno = 3
leaf.col_offset = 0
leaf.end_lineno = 3
leaf.end_col_offset = 1

expr = ast.Expr(leaf)
expr.lineno = 2
expr.col_offset = 0
expr.end_lineno = 2
expr.end_col_offset = 1

module = ast.Module([expr], [])
module.lineno = 1
module.col_offset = 0
module.end_lineno = 1
module.end_col_offset = 1

incremented = ast.increment_lineno(module, 5)

assert incremented is module
assert module.lineno == 1
assert module.col_offset == 0
assert module.end_lineno == 1
assert module.end_col_offset == 1
assert expr.lineno == 7
assert expr.col_offset == 0
assert expr.end_lineno == 7
assert expr.end_col_offset == 1
assert leaf.lineno == 8
assert leaf.col_offset == 0
assert leaf.end_lineno == 8
assert leaf.end_col_offset == 1

leaf.lineno = 10
leaf.end_lineno = None
ast.increment_lineno(leaf)
assert leaf.lineno == 11
assert leaf.end_lineno is None

op = ast.Add()
op.lineno = 4
op.end_lineno = 4
ast.increment_lineno(op, 5)
assert op.lineno == 4
assert op.end_lineno == 4
print("increment_lineno_updates_child_locations OK")
