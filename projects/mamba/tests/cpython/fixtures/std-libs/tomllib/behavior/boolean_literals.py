# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "boolean_literals"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: the literals true and false parse to the Python singletons True and False"""
import tomllib

_d = tomllib.loads("a = true\nb = false")
assert _d["a"] is True, f"true = {_d['a']!r}"
assert _d["b"] is False, f"false = {_d['b']!r}"

print("boolean_literals OK")
