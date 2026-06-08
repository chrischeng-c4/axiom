# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_level_names_mapping_is_present"
# subject = "logging.getLevelNamesMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getLevelNamesMapping: api_get_level_names_mapping_is_present (surface)."""
import logging

assert hasattr(logging, "getLevelNamesMapping")
print("api_get_level_names_mapping_is_present OK")
