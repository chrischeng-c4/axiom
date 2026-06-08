# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "dotted_value_matched_by_equality"
# subject = "match.value_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.value_pattern: dotted values (enum members, class attributes) are compared by equality"""

import enum


# Dotted values (enum members, class attributes) are compared by equality.
class Color(enum.Enum):
    RED = 0
    GREEN = 1
    BLUE = 2


def name_of(c):
    match c:
        case Color.RED:
            return "red"
        case Color.GREEN:
            return "green"
        case Color.BLUE:
            return "blue"
    return "unknown"


assert name_of(Color.RED) == "red"
assert name_of(Color.BLUE) == "blue"
assert name_of(99) == "unknown"
print("dotted_value_matched_by_equality OK")
