# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_re_is_present"
# subject = "platform.re"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.re: api_re_is_present (surface)."""
import platform

assert hasattr(platform, "re")
print("api_re_is_present OK")
