# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_buffer_flags_is_present"
# subject = "inspect.BufferFlags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.BufferFlags: api_buffer_flags_is_present (surface)."""
import inspect

assert hasattr(inspect, "BufferFlags")
print("api_buffer_flags_is_present OK")
