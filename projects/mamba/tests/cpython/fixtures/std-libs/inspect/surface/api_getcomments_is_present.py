# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getcomments_is_present"
# subject = "inspect.getcomments"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getcomments: api_getcomments_is_present (surface)."""
import inspect

assert hasattr(inspect, "getcomments")
print("api_getcomments_is_present OK")
