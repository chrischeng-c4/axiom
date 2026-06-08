# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_condition_is_present"
# subject = "threading.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.Condition: api_condition_is_present (surface)."""
import threading

assert hasattr(threading, "Condition")
print("api_condition_is_present OK")
