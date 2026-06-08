# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_uname_result_is_present"
# subject = "platform.uname_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.uname_result: api_uname_result_is_present (surface)."""
import platform

assert hasattr(platform, "uname_result")
print("api_uname_result_is_present OK")
