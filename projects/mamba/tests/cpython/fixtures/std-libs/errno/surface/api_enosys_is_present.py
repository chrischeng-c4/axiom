# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enosys_is_present"
# subject = "errno.ENOSYS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOSYS: api_enosys_is_present (surface)."""
import errno

assert hasattr(errno, "ENOSYS")
print("api_enosys_is_present OK")
