# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_supports_abs_is_present"
# subject = "typing.SupportsAbs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.SupportsAbs: api_supports_abs_is_present (surface)."""
import typing

assert hasattr(typing, "SupportsAbs")
print("api_supports_abs_is_present OK")
