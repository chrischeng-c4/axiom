# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "or_pattern_binds_same_name"
# subject = "match.or_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.or_pattern: OR patterns may bind names; the matching alternative supplies the binding"""

# OR patterns may bind names; the matching alternative supplies the binding.
match (2, 9):
    case (0 as v) | (v, 9):
        pass
assert v == 2  # second alternative matched, bound v to the first element
print("or_pattern_binds_same_name OK")
