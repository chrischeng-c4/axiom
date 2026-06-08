# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "surface"
# case = "import_graphlib"
# subject = "graphlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""graphlib: import_graphlib (surface)."""
import graphlib

assert hasattr(graphlib, "TopologicalSorter")
print("import_graphlib OK")
