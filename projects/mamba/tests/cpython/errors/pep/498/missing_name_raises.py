# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "missing_name_raises"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: missing_name_raises (errors)."""
# an unbound name in a field raises NameError at runtime

_raised = False
try:
    eval('f"v:{undefined_name}"')
except NameError:
    _raised = True
assert _raised, "missing_name_raises: expected NameError"
print("missing_name_raises OK")
