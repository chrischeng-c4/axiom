# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "singleton_pattern_parses"
# subject = "match.singleton_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.singleton_pattern: a singleton pattern (case None) parses, runs, and matches by identity"""

def probe(subject):
    match subject:
        case None:
            return "singleton"
    return "no-match"


assert probe(None) == "singleton"
assert probe(0) == "no-match"
print("singleton_pattern_parses OK")
