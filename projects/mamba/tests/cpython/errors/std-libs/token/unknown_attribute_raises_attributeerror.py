# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "errors"
# case = "unknown_attribute_raises_attributeerror"
# subject = "token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token: unknown_attribute_raises_attributeerror (errors)."""
import token

_raised = False
try:
    token.NO_SUCH_TOKEN_XYZZY
except AttributeError:
    _raised = True
assert _raised, "unknown_attribute_raises_attributeerror: expected AttributeError"
print("unknown_attribute_raises_attributeerror OK")
