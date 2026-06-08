# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getdoc_is_present"
# subject = "inspect.getdoc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getdoc: api_getdoc_is_present (surface)."""
import inspect

assert hasattr(inspect, "getdoc")
print("api_getdoc_is_present OK")
