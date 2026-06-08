# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_indentsize_is_present"
# subject = "inspect.indentsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.indentsize: api_indentsize_is_present (surface)."""
import inspect

assert hasattr(inspect, "indentsize")
print("api_indentsize_is_present OK")
