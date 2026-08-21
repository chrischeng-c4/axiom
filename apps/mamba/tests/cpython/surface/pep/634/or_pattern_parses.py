# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "or_pattern_parses"
# subject = "match.or_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.or_pattern: an OR pattern (case 'x' | 'y') parses, runs, and matches any alternative"""

def probe(subject):
    match subject:
        case "x" | "y":
            return "or"
    return "no-match"


assert probe("x") == "or"
assert probe("y") == "or"
assert probe("z") == "no-match"
print("or_pattern_parses OK")
