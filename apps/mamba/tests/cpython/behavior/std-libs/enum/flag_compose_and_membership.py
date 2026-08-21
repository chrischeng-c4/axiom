# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "flag_compose_and_membership"
# subject = "enum.Flag"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Flag: combining flags with | yields a composite whose .value is the bitwise-or; `in` reports component membership and a non-member bit is not in"""
import enum


class Color(enum.Flag):
    RED = enum.auto()
    BLUE = enum.auto()
    GREEN = enum.auto()


both = Color.RED | Color.BLUE
assert both.value == 3, f"composite value = {both.value!r}"
assert Color.RED in both, "RED is in the composite"
assert Color.BLUE in both, "BLUE is in the composite"
assert Color.GREEN not in both, "GREEN is not in the composite"

print("flag_compose_and_membership OK")
