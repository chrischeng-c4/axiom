# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_formatannotation_is_present"
# subject = "inspect.formatannotation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.formatannotation: api_formatannotation_is_present (surface)."""
import inspect

assert hasattr(inspect, "formatannotation")
print("api_formatannotation_is_present OK")
