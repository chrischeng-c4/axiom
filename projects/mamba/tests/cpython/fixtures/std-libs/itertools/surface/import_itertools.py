# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "import_itertools"
# subject = "itertools"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools: import_itertools (surface)."""
import itertools

assert hasattr(itertools, "count")
print("import_itertools OK")
