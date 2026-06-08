# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_is_dataclass_is_present"
# subject = "dataclasses.is_dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.is_dataclass: api_is_dataclass_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "is_dataclass")
print("api_is_dataclass_is_present OK")
