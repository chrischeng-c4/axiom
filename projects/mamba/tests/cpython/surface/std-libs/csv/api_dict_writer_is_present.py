# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_dict_writer_is_present"
# subject = "csv.DictWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.DictWriter: api_dict_writer_is_present (surface)."""
import csv

assert hasattr(csv, "DictWriter")
print("api_dict_writer_is_present OK")
