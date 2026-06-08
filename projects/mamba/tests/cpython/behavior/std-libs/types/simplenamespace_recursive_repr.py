# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_recursive_repr"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a self-referential SimpleNamespace renders its cycle as 'namespace(...)' instead of recursing forever"""
import types

loop = types.SimpleNamespace(c="cookie")
loop.spam = loop
assert repr(loop) == "namespace(c='cookie', spam=namespace(...))"

print("simplenamespace_recursive_repr OK")
