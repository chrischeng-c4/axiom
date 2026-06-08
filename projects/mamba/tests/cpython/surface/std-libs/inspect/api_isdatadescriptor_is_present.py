# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isdatadescriptor_is_present"
# subject = "inspect.isdatadescriptor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isdatadescriptor: api_isdatadescriptor_is_present (surface)."""
import inspect

assert hasattr(inspect, "isdatadescriptor")
print("api_isdatadescriptor_is_present OK")
