# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "errors"
# case = "missing_attribute_raises"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno: missing_attribute_raises (errors)."""
import errno

_raised = False
try:
    errno.NO_SUCH_ERRNO
except AttributeError:
    _raised = True
assert _raised, "missing_attribute_raises: expected AttributeError"
print("missing_attribute_raises OK")
