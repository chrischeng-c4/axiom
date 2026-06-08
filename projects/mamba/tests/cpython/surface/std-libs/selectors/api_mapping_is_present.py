# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_mapping_is_present"
# subject = "selectors.Mapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.Mapping: api_mapping_is_present (surface)."""
import selectors

assert hasattr(selectors, "Mapping")
print("api_mapping_is_present OK")
