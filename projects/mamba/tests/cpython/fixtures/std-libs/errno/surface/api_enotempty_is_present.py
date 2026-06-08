# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotempty_is_present"
# subject = "errno.ENOTEMPTY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTEMPTY: api_enotempty_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTEMPTY")
print("api_enotempty_is_present OK")
