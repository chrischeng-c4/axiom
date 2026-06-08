# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_win32_ver_is_present"
# subject = "platform.win32_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.win32_ver: api_win32_ver_is_present (surface)."""
import platform

assert hasattr(platform, "win32_ver")
print("api_win32_ver_is_present OK")
