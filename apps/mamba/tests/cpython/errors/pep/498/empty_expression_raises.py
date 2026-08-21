# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "empty_expression_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: empty_expression_raises (errors)."""
# an empty replacement field {} is a SyntaxError

_raised = False
try:
    eval('f"{}"')
except SyntaxError:
    _raised = True
assert _raised, "empty_expression_raises: expected SyntaxError"
print("empty_expression_raises OK")
