# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "noarg_construction_succeeds"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""graphlib.TopologicalSorter: TopologicalSorter is callable and a no-arg construction succeeds, yielding a non-None object on both runtimes"""
import graphlib

assert callable(graphlib.TopologicalSorter), "TopologicalSorter must be callable"
ts = graphlib.TopologicalSorter()
assert ts is not None, "no-arg construction yields an object"

print("noarg_construction_succeeds OK")
