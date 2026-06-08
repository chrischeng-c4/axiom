# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_findsource_is_present"
# subject = "inspect.findsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.findsource: api_findsource_is_present (surface)."""
import inspect

assert hasattr(inspect, "findsource")
print("api_findsource_is_present OK")
