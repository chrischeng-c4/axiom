# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_pop_only_front_map"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: pop/popitem consider only the first map (pop returns the default when the key is gone), and popitem on a drained front map raises KeyError"""
from collections import ChainMap

cm = ChainMap(dict(a=1, b=2), dict(c=30))
assert cm.pop("a", 1001) == 1, "pop an existing front-map key"
assert cm.pop("a", 1002) == 1002, "pop default once the key is gone"
try:
    cm.pop("c")  # c lives only in the back map
    raise AssertionError("expected KeyError popping a back-map key")
except KeyError:
    pass
drain = ChainMap(dict(a=1, b=2), dict(b=20, c=30))
drain.popitem()
drain.popitem()
try:
    drain.popitem()
    raise AssertionError("expected KeyError when the front map is drained")
except KeyError:
    pass

print("chainmap_pop_only_front_map OK")
