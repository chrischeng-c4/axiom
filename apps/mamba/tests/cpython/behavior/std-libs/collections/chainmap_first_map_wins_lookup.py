# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_first_map_wins_lookup"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: ChainMap looks up keys front-to-back so the first map shadows later ones; coercion to dict and .items() flatten with first-map-wins precedence"""
from collections import ChainMap

cm = ChainMap(dict(a=1, b=2), dict(b=20, c=30))
assert cm["a"] == 1 and cm["b"] == 2 and cm["c"] == 30, "front map shadows later maps"
assert dict(cm) == dict(a=1, b=2, c=30), f"flatten = {dict(cm)!r}"
assert dict(cm.items()) == dict(a=1, b=2, c=30), "items flatten with first-map-wins"

print("chainmap_first_map_wins_lookup OK")
