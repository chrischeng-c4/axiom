# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_itertools_is_present"
# subject = "platform.itertools"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.itertools: api_itertools_is_present (surface)."""
import platform

assert hasattr(platform, "itertools")
print("api_itertools_is_present OK")
