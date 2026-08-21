# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "wildcard_catches_anything"
# subject = "match.wildcard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.wildcard_pattern: the wildcard pattern (case _) matches any unmatched subject"""

# The wildcard pattern matches any unmatched subject.
def with_wildcard(x):
    match x:
        case 1:
            return "one"
        case _:
            return "other"


assert with_wildcard(1) == "one"
assert with_wildcard("anything") == "other"
print("wildcard_catches_anything OK")
