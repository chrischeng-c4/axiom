# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "bool_literal_distinct_from_int"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: True matches the True literal and 1 the int literal even though 1 == True; False reaches 0-case order"""

# bool is matched by literal identity; True is not the int pattern 1.
def truthy(x):
    match x:
        case True:
            return "true-lit"
        case 1:
            return "one-lit"
    return "none"


assert truthy(True) == "true-lit"
assert truthy(1) == "one-lit"  # 1 is not True even though 1 == True

# False reaches the 0-case only when 0 is written first; case order matters.
def zeroish(x):
    match x:
        case False:
            return "false-lit"
        case 0:
            return "zero-lit"
    return "none"


assert zeroish(0) == "zero-lit"  # 0 reaches the 0-case, not the False-case
assert zeroish(False) == "false-lit"
print("bool_literal_distinct_from_int OK")
