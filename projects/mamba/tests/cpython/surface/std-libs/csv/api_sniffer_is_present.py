# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_sniffer_is_present"
# subject = "csv.Sniffer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.Sniffer: api_sniffer_is_present (surface)."""
import csv

assert hasattr(csv, "Sniffer")
print("api_sniffer_is_present OK")
