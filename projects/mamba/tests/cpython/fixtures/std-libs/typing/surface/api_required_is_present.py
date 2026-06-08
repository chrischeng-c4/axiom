# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_required_is_present"
# subject = "typing.Required"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Required: api_required_is_present (surface)."""
import typing

assert hasattr(typing, "Required")
print("api_required_is_present OK")
