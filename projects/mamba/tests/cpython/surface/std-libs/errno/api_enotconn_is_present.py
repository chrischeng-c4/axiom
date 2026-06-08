# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotconn_is_present"
# subject = "errno.ENOTCONN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTCONN: api_enotconn_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTCONN")
print("api_enotconn_is_present OK")
