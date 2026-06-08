# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotrecoverable_is_present"
# subject = "errno.ENOTRECOVERABLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTRECOVERABLE: api_enotrecoverable_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTRECOVERABLE")
print("api_enotrecoverable_is_present OK")
