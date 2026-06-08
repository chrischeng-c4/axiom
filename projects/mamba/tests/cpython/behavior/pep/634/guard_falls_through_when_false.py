# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "guard_falls_through_when_false"
# subject = "match.guard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.guard_pattern: guards run only after the pattern matches; a false guard falls through to the next case"""

# Guards run only after the pattern matches; a false guard falls through.
def guarded(x):
    match x:
        case n if n < 0:
            return "neg"
        case n if n == 0:
            return "zero"
        case n:
            return "pos"


assert guarded(-3) == "neg"
assert guarded(0) == "zero"
assert guarded(5) == "pos"
print("guard_falls_through_when_false OK")
