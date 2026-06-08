# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_optional_is_present"
# subject = "typing.Optional"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Optional: api_optional_is_present (surface)."""
import typing

assert hasattr(typing, "Optional")
print("api_optional_is_present OK")
