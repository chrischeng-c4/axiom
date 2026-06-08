# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadf_is_present"
# subject = "errno.EBADF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADF: api_ebadf_is_present (surface)."""
import errno

assert hasattr(errno, "EBADF")
print("api_ebadf_is_present OK")
