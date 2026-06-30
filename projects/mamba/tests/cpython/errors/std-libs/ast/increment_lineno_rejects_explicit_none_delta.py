# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "errors"
# case = "increment_lineno_rejects_explicit_none_delta"
# subject = "ast.increment_lineno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
"""ast.increment_lineno: explicit None delta raises TypeError."""
import ast

node = ast.Constant(1)
node.lineno = 10
node.end_lineno = 10

raised = False
try:
    ast.increment_lineno(node, None)
except TypeError:
    raised = True

assert raised, "ast.increment_lineno(node, None) must raise TypeError"
assert node.lineno == 10
assert node.end_lineno == 10
print("increment_lineno_rejects_explicit_none_delta OK")
