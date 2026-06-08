# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_python_version_is_present"
# subject = "platform.python_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.python_version: api_python_version_is_present (surface)."""
import platform

assert hasattr(platform, "python_version")
print("api_python_version_is_present OK")
