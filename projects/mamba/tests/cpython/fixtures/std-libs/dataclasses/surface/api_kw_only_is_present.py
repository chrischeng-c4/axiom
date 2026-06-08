# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_kw_only_is_present"
# subject = "dataclasses.KW_ONLY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.KW_ONLY: api_kw_only_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "KW_ONLY")
print("api_kw_only_is_present OK")
