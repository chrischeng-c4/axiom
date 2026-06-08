# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_supports_int_is_present"
# subject = "typing.SupportsInt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.SupportsInt: api_supports_int_is_present (surface)."""
import typing

assert hasattr(typing, "SupportsInt")
print("api_supports_int_is_present OK")
