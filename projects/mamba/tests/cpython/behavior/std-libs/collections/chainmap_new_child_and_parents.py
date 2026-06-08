# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_new_child_and_parents"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: new_child pushes a fresh writable front map (writes/deletes touch only it, falling through to parents), and .parents drops the front map re-exposing inherited values"""
from collections import ChainMap

c = ChainMap()
c["a"] = 1
c["b"] = 2
d = c.new_child()
d["b"] = 20
d["c"] = 30
assert d.maps == [{"b": 20, "c": 30}, {"a": 1, "b": 2}], f"maps = {d.maps!r}"
assert d["a"] == 1 and d["b"] == 20 and len(d) == 3, "front map writes, parent fall-through"
del d["b"]
assert d.maps == [{"c": 30}, {"a": 1, "b": 2}], "del only touches the front map"
assert d["b"] == 2, "parent value re-exposed after del"
f = d.new_child()
f["b"] = 5
assert f["b"] == 5 and f.parents["b"] == 2, "parents drops the front map"

print("chainmap_new_child_and_parents OK")
