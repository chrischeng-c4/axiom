# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_ismethoddescriptor_is_present"
# subject = "inspect.ismethoddescriptor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ismethoddescriptor: api_ismethoddescriptor_is_present (surface)."""
import inspect

assert hasattr(inspect, "ismethoddescriptor")
print("api_ismethoddescriptor_is_present OK")
