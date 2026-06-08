# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadexec_is_present"
# subject = "errno.EBADEXEC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADEXEC: api_ebadexec_is_present (surface)."""
import errno

assert hasattr(errno, "EBADEXEC")
print("api_ebadexec_is_present OK")
