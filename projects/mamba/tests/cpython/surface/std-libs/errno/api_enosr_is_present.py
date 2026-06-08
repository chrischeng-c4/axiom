# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enosr_is_present"
# subject = "errno.ENOSR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOSR: api_enosr_is_present (surface)."""
import errno

assert hasattr(errno, "ENOSR")
print("api_enosr_is_present OK")
