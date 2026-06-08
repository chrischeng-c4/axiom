# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_formatargvalues_is_present"
# subject = "inspect.formatargvalues"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.formatargvalues: api_formatargvalues_is_present (surface)."""
import inspect

assert hasattr(inspect, "formatargvalues")
print("api_formatargvalues_is_present OK")
