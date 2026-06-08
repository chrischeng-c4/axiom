# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_architecture_is_present"
# subject = "platform.architecture"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.architecture: api_architecture_is_present (surface)."""
import platform

assert hasattr(platform, "architecture")
print("api_architecture_is_present OK")
