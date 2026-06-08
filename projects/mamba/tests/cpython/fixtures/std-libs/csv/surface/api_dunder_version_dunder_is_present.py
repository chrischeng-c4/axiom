# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_dunder_version_dunder_is_present"
# subject = "csv.__version__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.__version__: api_dunder_version_dunder_is_present (surface)."""
import csv

assert hasattr(csv, "__version__")
print("api_dunder_version_dunder_is_present OK")
