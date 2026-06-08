# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "surface"
# case = "class_positional_pattern_parses"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a positional class pattern (case Pt(px, py)) parses, runs, and binds via __match_args__"""

class Pt:
    __match_args__ = ("x", "y")

    def __init__(self, x, y):
        self.x = x
        self.y = y


def probe(subject):
    match subject:
        case Pt(px, py):
            return ("class", px, py)
    return "no-match"


assert probe(Pt(3, 4)) == ("class", 3, 4)
assert probe(0) == "no-match"
print("class_positional_pattern_parses OK")
