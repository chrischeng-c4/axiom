# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eloop_is_present"
# subject = "errno.ELOOP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ELOOP: api_eloop_is_present (surface)."""
import errno

assert hasattr(errno, "ELOOP")
print("api_eloop_is_present OK")
