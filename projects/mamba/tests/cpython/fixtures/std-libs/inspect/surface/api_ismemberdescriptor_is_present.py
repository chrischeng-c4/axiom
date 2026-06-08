# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_ismemberdescriptor_is_present"
# subject = "inspect.ismemberdescriptor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ismemberdescriptor: api_ismemberdescriptor_is_present (surface)."""
import inspect

assert hasattr(inspect, "ismemberdescriptor")
print("api_ismemberdescriptor_is_present OK")
