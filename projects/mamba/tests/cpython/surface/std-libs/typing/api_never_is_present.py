# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_never_is_present"
# subject = "typing.Never"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Never: api_never_is_present (surface)."""
import typing

assert hasattr(typing, "Never")
print("api_never_is_present OK")
