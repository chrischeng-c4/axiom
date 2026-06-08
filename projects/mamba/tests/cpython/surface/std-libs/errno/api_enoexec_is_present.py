# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enoexec_is_present"
# subject = "errno.ENOEXEC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOEXEC: api_enoexec_is_present (surface)."""
import errno

assert hasattr(errno, "ENOEXEC")
print("api_enoexec_is_present OK")
