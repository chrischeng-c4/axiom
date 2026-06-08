# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_self_is_present"
# subject = "typing.Self"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Self: api_self_is_present (surface)."""
import typing

assert hasattr(typing, "Self")
print("api_self_is_present OK")
