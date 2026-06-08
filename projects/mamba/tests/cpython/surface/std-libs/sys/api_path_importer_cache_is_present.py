# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_path_importer_cache_is_present"
# subject = "sys.path_importer_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.path_importer_cache: api_path_importer_cache_is_present (surface)."""
import sys

assert hasattr(sys, "path_importer_cache")
print("api_path_importer_cache_is_present OK")
