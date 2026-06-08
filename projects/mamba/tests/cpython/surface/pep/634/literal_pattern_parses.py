# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "literal_pattern_parses"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: a literal pattern (case 0) parses, runs, and matches by equality"""


def probe(subject):
    match subject:
        case 0:
            return "literal"
    return "no-match"


assert probe(0) == "literal"
assert probe(1) == "no-match"
print("literal_pattern_parses OK")
