# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_override_is_present"
# subject = "typing.override"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "apps/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.override: api_override_is_present (surface)."""
import typing

assert hasattr(typing, "override")
print("api_override_is_present OK")
