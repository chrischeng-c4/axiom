# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_stack_size_is_present"
# subject = "threading.stack_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.stack_size: api_stack_size_is_present (surface)."""
import threading

assert hasattr(threading, "stack_size")
print("api_stack_size_is_present OK")
