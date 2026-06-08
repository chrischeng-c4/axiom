# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isgenerator_is_present"
# subject = "inspect.isgenerator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isgenerator: api_isgenerator_is_present (surface)."""
import inspect

assert hasattr(inspect, "isgenerator")
print("api_isgenerator_is_present OK")
