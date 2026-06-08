# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enoattr_is_present"
# subject = "errno.ENOATTR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOATTR: api_enoattr_is_present (surface)."""
import errno

assert hasattr(errno, "ENOATTR")
print("api_enoattr_is_present OK")
