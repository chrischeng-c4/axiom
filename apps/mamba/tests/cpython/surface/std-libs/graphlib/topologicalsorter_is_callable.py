# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "surface"
# case = "topologicalsorter_is_callable"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""graphlib.TopologicalSorter: topologicalsorter_is_callable (surface)."""
import graphlib

assert callable(graphlib.TopologicalSorter)
print("topologicalsorter_is_callable OK")
