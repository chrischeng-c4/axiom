# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enolink_is_present"
# subject = "errno.ENOLINK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOLINK: api_enolink_is_present (surface)."""
import errno

assert hasattr(errno, "ENOLINK")
print("api_enolink_is_present OK")
