# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_formatannotationrelativeto_is_present"
# subject = "inspect.formatannotationrelativeto"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.formatannotationrelativeto: api_formatannotationrelativeto_is_present (surface)."""
import inspect

assert hasattr(inspect, "formatannotationrelativeto")
print("api_formatannotationrelativeto_is_present OK")
