# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "div_zero_propagates"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: div_zero_propagates (errors)."""
# expression failures inside a field propagate (builtin eval)

_raised = False
try:
    eval('f"{1/0}"')
except ZeroDivisionError:
    _raised = True
assert _raised, "div_zero_propagates: expected ZeroDivisionError"
print("div_zero_propagates OK")
