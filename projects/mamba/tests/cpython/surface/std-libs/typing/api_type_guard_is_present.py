# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_type_guard_is_present"
# subject = "typing.TypeGuard"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TypeGuard: api_type_guard_is_present (surface)."""
import typing

assert hasattr(typing, "TypeGuard")
print("api_type_guard_is_present OK")
