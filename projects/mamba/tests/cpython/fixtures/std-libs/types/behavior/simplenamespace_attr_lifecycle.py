# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_attr_lifecycle"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: attribute set / get / del lifecycle on a SimpleNamespace updates vars(), and deleting an absent attribute raises AttributeError"""
import types

ns = types.SimpleNamespace(a=1, b=2, c=3)
assert ns.a == 1
ns.d = "added"
assert ns.d == "added"
del ns.b
assert vars(ns) == {"a": 1, "c": 3, "d": "added"}

# Deleting an absent attribute raises AttributeError.
_raised = False
try:
    del ns.missing
except AttributeError:
    _raised = True
assert _raised, "deleting an absent attribute should raise AttributeError"

print("simplenamespace_attr_lifecycle OK")
