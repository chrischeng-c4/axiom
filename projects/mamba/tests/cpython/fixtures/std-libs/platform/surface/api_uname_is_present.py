# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_uname_is_present"
# subject = "platform.uname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.uname: api_uname_is_present (surface)."""
import platform

assert hasattr(platform, "uname")
print("api_uname_is_present OK")
