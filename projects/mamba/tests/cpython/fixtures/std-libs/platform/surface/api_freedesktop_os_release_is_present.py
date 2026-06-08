# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_freedesktop_os_release_is_present"
# subject = "platform.freedesktop_os_release"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.freedesktop_os_release: api_freedesktop_os_release_is_present (surface)."""
import platform

assert hasattr(platform, "freedesktop_os_release")
print("api_freedesktop_os_release_is_present OK")
