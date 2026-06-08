# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ewouldblock_is_present"
# subject = "errno.EWOULDBLOCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EWOULDBLOCK: api_ewouldblock_is_present (surface)."""
import errno

assert hasattr(errno, "EWOULDBLOCK")
print("api_ewouldblock_is_present OK")
