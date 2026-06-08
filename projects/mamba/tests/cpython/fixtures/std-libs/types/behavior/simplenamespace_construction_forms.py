# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_construction_forms"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: empty(), kwargs, and **dict construction are equivalent forms whose vars()/__dict__ reflect the supplied attributes"""
import types

ns_empty = types.SimpleNamespace()
ns_kw = types.SimpleNamespace(x=1, y=2)
ns_unpack = types.SimpleNamespace(**dict(x=1, y=2))
assert vars(ns_empty) == {}
assert vars(ns_kw) == {"x": 1, "y": 2}
assert ns_unpack.__dict__ == {"x": 1, "y": 2}
assert len(ns_kw.__dict__) == 2

print("simplenamespace_construction_forms OK")
