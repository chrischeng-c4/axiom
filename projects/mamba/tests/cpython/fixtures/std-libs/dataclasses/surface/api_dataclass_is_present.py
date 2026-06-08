# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_dataclass_is_present"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.dataclass: api_dataclass_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "dataclass")
print("api_dataclass_is_present OK")
