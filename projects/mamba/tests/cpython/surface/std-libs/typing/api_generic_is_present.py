# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_generic_is_present"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Generic: api_generic_is_present (surface)."""
import typing

assert hasattr(typing, "Generic")
print("api_generic_is_present OK")
