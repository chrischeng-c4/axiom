# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotsock_is_present"
# subject = "errno.ENOTSOCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTSOCK: api_enotsock_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTSOCK")
print("api_enotsock_is_present OK")
