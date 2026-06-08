# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadrpc_is_present"
# subject = "errno.EBADRPC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADRPC: api_ebadrpc_is_present (surface)."""
import errno

assert hasattr(errno, "EBADRPC")
print("api_ebadrpc_is_present OK")
