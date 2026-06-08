# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_sized_is_present"
# subject = "typing.Sized"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Sized: api_sized_is_present (surface)."""
import typing

assert hasattr(typing, "Sized")
print("api_sized_is_present OK")
