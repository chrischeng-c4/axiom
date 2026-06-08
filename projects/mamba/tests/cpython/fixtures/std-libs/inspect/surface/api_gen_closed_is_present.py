# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_gen_closed_is_present"
# subject = "inspect.GEN_CLOSED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.GEN_CLOSED: api_gen_closed_is_present (surface)."""
import inspect

assert hasattr(inspect, "GEN_CLOSED")
print("api_gen_closed_is_present OK")
