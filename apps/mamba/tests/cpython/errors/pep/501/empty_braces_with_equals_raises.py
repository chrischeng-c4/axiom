# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "errors"
# case = "empty_braces_with_equals_raises"
# subject = "fstring.debug_equals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: empty_braces_with_equals_raises (errors)."""
pass

_raised = False
try:
    eval('f"{ = }"')
except SyntaxError:
    _raised = True
assert _raised, "empty_braces_with_equals_raises: expected SyntaxError"
print("empty_braces_with_equals_raises OK")
