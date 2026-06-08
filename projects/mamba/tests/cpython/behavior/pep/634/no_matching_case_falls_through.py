# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "no_matching_case_falls_through"
# subject = "match.statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.statement: a match with no matching case and no wildcard falls through without raising"""

# A match with no matching case and no wildcard falls through (no raise).
def classify(x):
    match x:
        case 1:
            return "one"
        case "two":
            return "two"
    return "unmatched"


assert classify(1) == "one"
assert classify("two") == "two"
assert classify(99) == "unmatched"
print("no_matching_case_falls_through OK")
