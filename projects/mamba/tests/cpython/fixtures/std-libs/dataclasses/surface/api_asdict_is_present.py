# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_asdict_is_present"
# subject = "dataclasses.asdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.asdict: api_asdict_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "asdict")
print("api_asdict_is_present OK")
