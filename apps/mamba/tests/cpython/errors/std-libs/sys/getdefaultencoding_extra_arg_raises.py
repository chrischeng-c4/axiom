# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "getdefaultencoding_extra_arg_raises"
# subject = "sys.getdefaultencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getdefaultencoding: getdefaultencoding_extra_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.getdefaultencoding(42)
except TypeError:
    _raised = True
assert _raised, "getdefaultencoding_extra_arg_raises: expected TypeError"
print("getdefaultencoding_extra_arg_raises OK")
