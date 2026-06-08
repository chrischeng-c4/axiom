# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "star_captures_middle_as_list"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a star target captures the middle/leading/trailing slice as a list in every position"""

# A star target captures the slice as a list, in every position.
match (0, 1, 2, 3):
    case [first, *middle, last]:
        pass
assert first == 0 and last == 3 and middle == [1, 2]

match (0, 1, 2):
    case [*rest, 2]:
        pass
assert rest == [0, 1]

match (0, 1, 2):
    case [0, 1, 2, *tailrest]:
        pass
assert tailrest == []
print("star_captures_middle_as_list OK")
