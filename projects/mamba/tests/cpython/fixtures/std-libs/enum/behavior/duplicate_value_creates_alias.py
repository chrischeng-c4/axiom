# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "duplicate_value_creates_alias"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a second name bound to an existing value becomes an alias of the first member (same singleton) and does not appear in iteration"""
import enum


class Weekday(enum.Enum):
    MON = 1
    TUE = 2
    WED = 3
    MON2 = 1  # alias for MON


assert Weekday.MON2 is Weekday.MON, "duplicate value -> alias is the same member"
assert len(list(Weekday)) == 3, f"aliases excluded from iteration: {len(list(Weekday))!r}"

print("duplicate_value_creates_alias OK")
