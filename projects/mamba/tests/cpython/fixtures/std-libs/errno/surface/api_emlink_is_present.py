# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_emlink_is_present"
# subject = "errno.EMLINK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EMLINK: api_emlink_is_present (surface)."""
import errno

assert hasattr(errno, "EMLINK")
print("api_emlink_is_present OK")
