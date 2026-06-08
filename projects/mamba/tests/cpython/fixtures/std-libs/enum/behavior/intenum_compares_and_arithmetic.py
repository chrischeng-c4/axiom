# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "intenum_compares_and_arithmetic"
# subject = "enum.IntEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: IntEnum members are real ints: they compare to ints and to each other, and arithmetic on them returns a plain int"""
import enum


class Score(enum.IntEnum):
    LOW = 1
    MID = 5
    HIGH = 10


# IntEnum members compare to ints and to one another.
assert Score.MID == 5, "IntEnum member equals its int value"
assert Score.HIGH > 5, "IntEnum compares to a bare int"
assert Score.HIGH > Score.LOW, "IntEnum members compare to each other"
assert isinstance(Score.HIGH, int), f"IntEnum is int: {type(Score.HIGH)!r}"

# Arithmetic on IntEnum members returns a plain int, not an enum member.
total = Score.MID + Score.LOW
assert total == 6, f"IntEnum add = {total!r}"
assert isinstance(total, int) and not isinstance(total, Score), "add returns plain int"

print("intenum_compares_and_arithmetic OK")
