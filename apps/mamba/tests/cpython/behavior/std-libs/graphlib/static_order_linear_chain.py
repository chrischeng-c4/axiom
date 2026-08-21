# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "static_order_linear_chain"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: static_order on the chain A<-B<-C ({'A':{'B'},'B':{'C'},'C':set()}) yields dependencies before dependents: ['C','B','A']"""
import graphlib

ts = graphlib.TopologicalSorter({"A": {"B"}, "B": {"C"}, "C": set()})
order = list(ts.static_order())
assert order == ["C", "B", "A"], order

print("static_order_linear_chain OK")
