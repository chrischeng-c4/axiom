# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_java_ver_is_present"
# subject = "platform.java_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.java_ver: api_java_ver_is_present (surface)."""
import platform

assert hasattr(platform, "java_ver")
print("api_java_ver_is_present OK")
