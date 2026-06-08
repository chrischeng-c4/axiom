# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getmembers_returns_name_value_tuples"
# subject = "inspect.getmembers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getmembers: getmembers() returns a list of (name, value) tuples; with a predicate it filters them"""
import inspect
import math

_members = inspect.getmembers(math, predicate=inspect.isbuiltin)
assert isinstance(_members, list), f"getmembers type = {type(_members)!r}"
assert len(_members) > 0, "math has builtin members"
assert all(isinstance(m, tuple) for m in _members), "members are tuples"

class _Sample:
    x = 1
    def method(self):
        return 2

_d = dict(inspect.getmembers(_Sample))
assert "x" in _d, "x in members"
assert "method" in _d, "method in members"
assert _d["x"] == 1, f"member x = {_d['x']!r}"

print("getmembers_returns_name_value_tuples OK")
