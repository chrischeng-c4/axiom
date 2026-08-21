# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "capture_vs_wildcard_binding"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a capture pattern binds the subject to the name; a literal case before it takes precedence"""

# A capture pattern binds the subject; an earlier literal case takes precedence.
def label(x):
    match x:
        case 0:
            return "zero"
        case other:
            return ("captured", other)


assert label(0) == "zero"
assert label(99) == ("captured", 99)
print("capture_vs_wildcard_binding OK")
