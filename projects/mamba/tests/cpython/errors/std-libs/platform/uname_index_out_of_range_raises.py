# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "uname_index_out_of_range_raises"
# subject = "platform.uname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.uname: uname_index_out_of_range_raises (errors)."""
import platform

_raised = False
try:
    platform.uname()[6]
except IndexError:
    _raised = True
assert _raised, "uname_index_out_of_range_raises: expected IndexError"
print("uname_index_out_of_range_raises OK")
