# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "builtin_single_positional_binds_whole"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a single positional pattern against a builtin (bool/int/str) binds the whole subject; bool before int"""

# A single positional pattern against a builtin type binds the whole subject.
def kind(x):
    match x:
        case bool(b):
            return ("bool", b)
        case int(n):
            return ("int", n)
        case str(s):
            return ("str", s)
    return "other"


assert kind(True) == ("bool", True)  # bool checked before int
assert kind(7) == ("int", 7)
assert kind("hi") == ("str", "hi")
assert kind(1.5) == "other"
print("builtin_single_positional_binds_whole OK")
