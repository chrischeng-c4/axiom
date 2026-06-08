# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_not_required_is_present"
# subject = "typing.NotRequired"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.NotRequired: api_not_required_is_present (surface)."""
import typing

assert hasattr(typing, "NotRequired")
print("api_not_required_is_present OK")
