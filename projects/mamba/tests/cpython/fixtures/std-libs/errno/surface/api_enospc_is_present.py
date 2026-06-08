# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enospc_is_present"
# subject = "errno.ENOSPC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOSPC: api_enospc_is_present (surface)."""
import errno

assert hasattr(errno, "ENOSPC")
print("api_enospc_is_present OK")
