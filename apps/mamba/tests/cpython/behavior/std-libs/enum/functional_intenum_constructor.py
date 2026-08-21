# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "functional_intenum_constructor"
# subject = "enum.IntEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: IntEnum('Codes', 'ok created accepted') yields int-comparable members numbered from 1"""
import enum

Codes = enum.IntEnum("Codes", "ok created accepted")

assert Codes.ok == 1, "IntEnum functional value numbered from 1"
assert Codes.accepted == 3, "IntEnum functional value in order"
assert isinstance(Codes.ok, int), "functional IntEnum member is int"

print("functional_intenum_constructor OK")
