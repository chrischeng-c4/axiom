# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_repr"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: repr renders 'namespace(...)' with attributes in insertion order, including underscored names"""
import types

assert repr(types.SimpleNamespace(x=1, y=2, w=3)) == "namespace(x=1, y=2, w=3)"
spammy = types.SimpleNamespace()
spammy.x = "spam"
spammy._y = 5
assert repr(spammy) == "namespace(x='spam', _y=5)"

print("simplenamespace_repr OK")
