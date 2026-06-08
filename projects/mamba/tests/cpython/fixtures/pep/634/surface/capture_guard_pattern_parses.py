# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "capture_guard_pattern_parses"
# subject = "match.guard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.guard_pattern: a capture pattern with a guard (case n if n > 100) parses, runs, and binds the subject"""

def probe(subject):
    match subject:
        case n if n > 100:
            return ("guard", n)
    return "no-match"


assert probe(500) == ("guard", 500)
assert probe(50) == "no-match"
print("capture_guard_pattern_parses OK")
