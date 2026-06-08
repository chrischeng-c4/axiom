# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_gen_running_is_present"
# subject = "inspect.GEN_RUNNING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.GEN_RUNNING: api_gen_running_is_present (surface)."""
import inspect

assert hasattr(inspect, "GEN_RUNNING")
print("api_gen_running_is_present OK")
