# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_libc_ver_is_present"
# subject = "platform.libc_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.libc_ver: api_libc_ver_is_present (surface)."""
import platform

assert hasattr(platform, "libc_ver")
print("api_libc_ver_is_present OK")
