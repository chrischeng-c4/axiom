# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotblk_is_present"
# subject = "errno.ENOTBLK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTBLK: api_enotblk_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTBLK")
print("api_enotblk_is_present OK")
