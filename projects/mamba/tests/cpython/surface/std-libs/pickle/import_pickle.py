# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "import_pickle"
# subject = "pickle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle: import_pickle (surface)."""
import pickle

assert hasattr(pickle, "dumps")
print("import_pickle OK")
