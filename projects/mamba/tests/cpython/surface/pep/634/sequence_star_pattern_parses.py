# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "sequence_star_pattern_parses"
# subject = "match.sequence_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.sequence_pattern: a sequence pattern with a star (case [a, *_]) parses, runs, and captures the head"""

def probe(subject):
    match subject:
        case [a, *_]:
            return ("sequence", a)
    return "no-match"


assert probe([1, 2, 3]) == ("sequence", 1)
assert probe("abc") == "no-match"
print("sequence_star_pattern_parses OK")
