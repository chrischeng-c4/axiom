# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "intern_returns_same_object"
# subject = "sys.intern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.intern: interning the same string twice returns the identical object (a is b)"""
import sys

_a = sys.intern("hello_world_unique_key")
_b = sys.intern("hello_world_unique_key")
assert _a is _b, "interned strings are same object"
print("intern_returns_same_object OK")
