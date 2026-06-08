# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_positional_and_keyword_subpatterns"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: positional class patterns read __match_args__; keyword subpatterns bind by attribute name"""


# Positional class patterns read __match_args__; keyword subpatterns bind by name.
class Point:
    __match_args__ = ("x", "y")

    def __init__(self, x, y):
        self.x = x
        self.y = y


def describe(p):
    match p:
        case Point(0, 0):
            return "origin"
        case Point(x=0, y=yy):
            return ("on-y", yy)
        case Point(a, b):
            return ("point", a, b)
    return "not-point"


assert describe(Point(0, 0)) == "origin"
assert describe(Point(0, 5)) == ("on-y", 5)  # keyword subpattern
assert describe(Point(1, 2)) == ("point", 1, 2)
assert describe("nope") == "not-point"
print("class_positional_and_keyword_subpatterns OK")
