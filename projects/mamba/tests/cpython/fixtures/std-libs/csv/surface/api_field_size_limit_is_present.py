# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_field_size_limit_is_present"
# subject = "csv.field_size_limit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.field_size_limit: api_field_size_limit_is_present (surface)."""
import csv

assert hasattr(csv, "field_size_limit")
print("api_field_size_limit_is_present OK")
