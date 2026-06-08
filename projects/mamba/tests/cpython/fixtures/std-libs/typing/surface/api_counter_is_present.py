# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_counter_is_present"
# subject = "typing.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Counter: api_counter_is_present (surface)."""
import typing

assert hasattr(typing, "Counter")
print("api_counter_is_present OK")
