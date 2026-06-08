# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isgetsetdescriptor_is_present"
# subject = "inspect.isgetsetdescriptor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isgetsetdescriptor: api_isgetsetdescriptor_is_present (surface)."""
import inspect

assert hasattr(inspect, "isgetsetdescriptor")
print("api_isgetsetdescriptor_is_present OK")
