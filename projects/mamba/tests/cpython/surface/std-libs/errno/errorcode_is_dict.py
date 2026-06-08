# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "errorcode_is_dict"
# subject = "errno.errorcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode_is_dict (surface)."""
import errno

assert type(errno.errorcode).__name__ == "dict"
print("errorcode_is_dict OK")
