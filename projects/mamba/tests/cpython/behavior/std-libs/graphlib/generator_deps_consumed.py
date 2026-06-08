# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "generator_deps_consumed"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: dependencies may be any iterable including a one-shot generator: {0: (2*x+1 for x in range(5))} gives static_order [1,3,5,7,9,0]"""
import graphlib

deps = (2 * x + 1 for x in range(5))
gen_ts = graphlib.TopologicalSorter({0: deps})
gen_order = list(gen_ts.static_order())
assert gen_order == [1, 3, 5, 7, 9, 0], gen_order

print("generator_deps_consumed OK")
