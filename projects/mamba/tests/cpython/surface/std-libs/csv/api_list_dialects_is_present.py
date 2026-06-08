# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_list_dialects_is_present"
# subject = "csv.list_dialects"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.list_dialects: api_list_dialects_is_present (surface)."""
import csv

assert hasattr(csv, "list_dialects")
print("api_list_dialects_is_present OK")
