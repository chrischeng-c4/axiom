# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_frozen_set_is_present"
# subject = "typing.FrozenSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.FrozenSet: api_frozen_set_is_present (surface)."""
import typing

assert hasattr(typing, "FrozenSet")
print("api_frozen_set_is_present OK")
