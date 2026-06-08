# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_dunder_doc_dunder_is_present"
# subject = "csv.__doc__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.__doc__: api_dunder_doc_dunder_is_present (surface)."""
import csv

assert hasattr(csv, "__doc__")
print("api_dunder_doc_dunder_is_present OK")
