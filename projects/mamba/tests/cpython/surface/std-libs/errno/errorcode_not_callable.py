# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "errorcode_not_callable"
# subject = "errno.errorcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode_not_callable (surface)."""
import errno

assert not callable(errno.errorcode)
print("errorcode_not_callable OK")
