# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_efault_is_present"
# subject = "errno.EFAULT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EFAULT: api_efault_is_present (surface)."""
import errno

assert hasattr(errno, "EFAULT")
print("api_efault_is_present OK")
