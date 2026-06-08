# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_equality"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: SimpleNamespace equality compares the underlying __dict__: two namespaces with equal attrs are ==, two empty ones are ==, and an empty differs from a populated one"""
import types

left = types.SimpleNamespace(x=1)
right = types.SimpleNamespace()
right.x = 1
assert left == right
assert types.SimpleNamespace() == types.SimpleNamespace()
assert right != types.SimpleNamespace()

print("simplenamespace_equality OK")
