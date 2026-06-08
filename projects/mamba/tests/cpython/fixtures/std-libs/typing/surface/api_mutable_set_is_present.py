# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_mutable_set_is_present"
# subject = "typing.MutableSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.MutableSet: api_mutable_set_is_present (surface)."""
import typing

assert hasattr(typing, "MutableSet")
print("api_mutable_set_is_present OK")
