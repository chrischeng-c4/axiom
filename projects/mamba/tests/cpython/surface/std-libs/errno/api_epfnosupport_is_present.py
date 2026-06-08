# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_epfnosupport_is_present"
# subject = "errno.EPFNOSUPPORT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EPFNOSUPPORT: api_epfnosupport_is_present (surface)."""
import errno

assert hasattr(errno, "EPFNOSUPPORT")
print("api_epfnosupport_is_present OK")
