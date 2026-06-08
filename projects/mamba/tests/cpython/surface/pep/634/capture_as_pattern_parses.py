# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "capture_as_pattern_parses"
# subject = "match.as_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.as_pattern: an AS capture pattern (case other as kept) parses, runs, and binds the whole value"""

def probe(subject):
    match subject:
        case other as kept:
            return ("capture", kept)
    return "no-match"


assert probe(7) == ("capture", 7)
print("capture_as_pattern_parses OK")
