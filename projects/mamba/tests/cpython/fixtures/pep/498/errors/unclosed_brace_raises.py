# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "unclosed_brace_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: unclosed_brace_raises (errors)."""
# an unclosed replacement-field brace is a SyntaxError

_raised = False
try:
    eval('f"hello {x"')
except SyntaxError:
    _raised = True
assert _raised, "unclosed_brace_raises: expected SyntaxError"
print("unclosed_brace_raises OK")
