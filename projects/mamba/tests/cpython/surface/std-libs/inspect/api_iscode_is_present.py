# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_iscode_is_present"
# subject = "inspect.iscode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.iscode: api_iscode_is_present (surface)."""
import inspect

assert hasattr(inspect, "iscode")
print("api_iscode_is_present OK")
