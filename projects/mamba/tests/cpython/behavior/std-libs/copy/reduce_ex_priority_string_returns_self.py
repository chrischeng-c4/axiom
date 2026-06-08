# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_ex_priority_string_returns_self"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: __reduce_ex__ is consulted once with protocol 4 and takes priority over __reduce__; a string result means copy returns the object itself"""
import copy

calls = []


class ReduceEx:
    def __reduce_ex__(self, proto):
        calls.append(proto)
        return ""  # a string result -> copy returns the object itself

    def __reduce__(self):
        raise AssertionError("__reduce__ should not be consulted")


rx = ReduceEx()
assert copy.copy(rx) is rx, "reduce_ex string result returns self"
assert calls == [4], f"reduce_ex called once with protocol 4, got {calls!r}"

print("reduce_ex_priority_string_returns_self OK")
