# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_pickle_by_global_name_is_present"
# subject = "enum.pickle_by_global_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.pickle_by_global_name: api_pickle_by_global_name_is_present (surface)."""
import enum

assert hasattr(enum, "pickle_by_global_name")
print("api_pickle_by_global_name_is_present OK")
