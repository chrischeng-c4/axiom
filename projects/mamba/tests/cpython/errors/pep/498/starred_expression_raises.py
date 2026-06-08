# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "starred_expression_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: starred_expression_raises (errors)."""
# a starred expression is not a valid f-string field (SyntaxError)

_raised = False
try:
    compile("f'{*a}'", '?', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "starred_expression_raises: expected SyntaxError"
print("starred_expression_raises OK")
