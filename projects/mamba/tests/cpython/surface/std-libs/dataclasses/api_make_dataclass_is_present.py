# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_make_dataclass_is_present"
# subject = "dataclasses.make_dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.make_dataclass: api_make_dataclass_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "make_dataclass")
print("api_make_dataclass_is_present OK")
