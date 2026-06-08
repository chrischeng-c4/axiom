# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_edestaddrreq_is_present"
# subject = "errno.EDESTADDRREQ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EDESTADDRREQ: api_edestaddrreq_is_present (surface)."""
import errno

assert hasattr(errno, "EDESTADDRREQ")
print("api_edestaddrreq_is_present OK")
