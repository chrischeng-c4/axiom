# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_functools_is_present"
# subject = "platform.functools"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.functools: api_functools_is_present (surface)."""
import platform

assert hasattr(platform, "functools")
print("api_functools_is_present OK")
