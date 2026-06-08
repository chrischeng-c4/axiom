# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_errorcode_is_present"
# subject = "errno.errorcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.errorcode: api_errorcode_is_present (surface)."""
import errno

assert hasattr(errno, "errorcode")
print("api_errorcode_is_present OK")
