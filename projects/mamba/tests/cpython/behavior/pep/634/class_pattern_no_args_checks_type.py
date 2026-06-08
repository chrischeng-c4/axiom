# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_pattern_no_args_checks_type"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a class pattern with no subpatterns just checks the type; first matching case wins"""

# A class pattern with no subpatterns just checks the type; first match wins.
def first_match_wins(x):
    match x:
        case int():
            return "int"
        case object():
            return "object"


assert first_match_wins(3) == "int"
assert first_match_wins("s") == "object"
print("class_pattern_no_args_checks_type OK")
