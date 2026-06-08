# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_mac_ver_is_present"
# subject = "platform.mac_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.mac_ver: api_mac_ver_is_present (surface)."""
import platform

assert hasattr(platform, "mac_ver")
print("api_mac_ver_is_present OK")
