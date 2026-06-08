# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_agen_created_is_present"
# subject = "inspect.AGEN_CREATED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.AGEN_CREATED: api_agen_created_is_present (surface)."""
import inspect

assert hasattr(inspect, "AGEN_CREATED")
print("api_agen_created_is_present OK")
