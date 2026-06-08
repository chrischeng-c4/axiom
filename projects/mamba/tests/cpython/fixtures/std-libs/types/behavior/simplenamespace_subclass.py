# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_subclass"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a SimpleNamespace subclass keeps its own type while reusing the kwargs attribute machinery and vars() view"""
import types


class Sub(types.SimpleNamespace):
    pass


sub = Sub(ham=8, eggs=9)
assert type(sub) is Sub
assert isinstance(sub, types.SimpleNamespace)
assert vars(sub) == {"ham": 8, "eggs": 9}

print("simplenamespace_subclass OK")
