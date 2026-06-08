# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eisconn_is_present"
# subject = "errno.EISCONN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EISCONN: api_eisconn_is_present (surface)."""
import errno

assert hasattr(errno, "EISCONN")
print("api_eisconn_is_present OK")
