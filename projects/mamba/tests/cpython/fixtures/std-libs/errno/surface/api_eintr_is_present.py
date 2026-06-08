# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eintr_is_present"
# subject = "errno.EINTR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EINTR: api_eintr_is_present (surface)."""
import errno

assert hasattr(errno, "EINTR")
print("api_eintr_is_present OK")
