# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eagain_is_present"
# subject = "errno.EAGAIN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EAGAIN: api_eagain_is_present (surface)."""
import errno

assert hasattr(errno, "EAGAIN")
print("api_eagain_is_present OK")
