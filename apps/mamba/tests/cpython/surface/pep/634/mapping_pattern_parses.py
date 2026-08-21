# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "mapping_pattern_parses"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a mapping pattern (case {'k': v}) parses, runs, and binds the value"""

def probe(subject):
    match subject:
        case {"k": v}:
            return ("mapping", v)
    return "no-match"


assert probe({"k": 9}) == ("mapping", 9)
assert probe({}) == "no-match"
print("mapping_pattern_parses OK")
