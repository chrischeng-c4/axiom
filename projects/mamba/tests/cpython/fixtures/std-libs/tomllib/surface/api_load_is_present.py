# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "api_load_is_present"
# subject = "tomllib.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tomllib.load: api_load_is_present (surface)."""
import tomllib

assert hasattr(tomllib, "load")
print("api_load_is_present OK")
