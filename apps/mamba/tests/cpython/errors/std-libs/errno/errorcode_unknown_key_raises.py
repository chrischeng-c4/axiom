# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "errors"
# case = "errorcode_unknown_key_raises"
# subject = "errno.errorcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode_unknown_key_raises (errors)."""
import errno

_raised = False
try:
    errno.errorcode[99999]
except KeyError:
    _raised = True
assert _raised, "errorcode_unknown_key_raises: expected KeyError"
print("errorcode_unknown_key_raises OK")
