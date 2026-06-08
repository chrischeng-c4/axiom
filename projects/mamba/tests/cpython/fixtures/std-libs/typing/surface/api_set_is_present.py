# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_set_is_present"
# subject = "typing.Set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Set: api_set_is_present (surface)."""
import typing

assert hasattr(typing, "Set")
print("api_set_is_present OK")
