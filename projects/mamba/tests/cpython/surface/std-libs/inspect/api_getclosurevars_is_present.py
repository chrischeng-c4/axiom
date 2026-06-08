# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getclosurevars_is_present"
# subject = "inspect.getclosurevars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getclosurevars: api_getclosurevars_is_present (surface)."""
import inspect

assert hasattr(inspect, "getclosurevars")
print("api_getclosurevars_is_present OK")
