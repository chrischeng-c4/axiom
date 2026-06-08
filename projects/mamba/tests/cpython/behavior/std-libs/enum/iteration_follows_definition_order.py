# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "iteration_follows_definition_order"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: list(Enum) yields members in definition order, not value order, and excludes aliases"""
import enum


class Season(enum.Enum):
    SUMMER = 2
    WINTER = 4
    AUTUMN = 3
    SPRING = 1


# Iteration follows definition order, not value order.
assert list(Season) == [Season.SUMMER, Season.WINTER, Season.AUTUMN, Season.SPRING], \
    f"order = {list(Season)!r}"

print("iteration_follows_definition_order OK")
