# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_agen_running_is_present"
# subject = "inspect.AGEN_RUNNING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.AGEN_RUNNING: api_agen_running_is_present (surface)."""
import inspect

assert hasattr(inspect, "AGEN_RUNNING")
print("api_agen_running_is_present OK")
