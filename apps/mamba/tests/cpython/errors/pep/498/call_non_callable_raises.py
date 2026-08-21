# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "call_non_callable_raises"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: call_non_callable_raises (errors)."""
# calling a non-callable inside a field raises TypeError

_raised = False
try:
    eval('f"{(1)()}"')
except TypeError:
    _raised = True
assert _raised, "call_non_callable_raises: expected TypeError"
print("call_non_callable_raises OK")
