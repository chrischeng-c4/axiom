# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "literal_case_binds_nothing"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: a literal-only case introduces no new names into scope"""

# A literal-only case introduces no new names into scope.
def literal_case_binds_nothing():
    seen_before = set(locals())
    match 1:
        case 1 | 2 | 3:
            pass
    return set(locals()) - seen_before - {"seen_before"}


assert literal_case_binds_nothing() == set()
print("literal_case_binds_nothing OK")
