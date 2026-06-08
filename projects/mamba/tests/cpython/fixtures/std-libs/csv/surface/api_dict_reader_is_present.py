# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_dict_reader_is_present"
# subject = "csv.DictReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.DictReader: api_dict_reader_is_present (surface)."""
import csv

assert hasattr(csv, "DictReader")
print("api_dict_reader_is_present OK")
