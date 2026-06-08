# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "api_loads_is_present"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tomllib.loads: api_loads_is_present (surface)."""
import tomllib

assert hasattr(tomllib, "loads")
print("api_loads_is_present OK")
