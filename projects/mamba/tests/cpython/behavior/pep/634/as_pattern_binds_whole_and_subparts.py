# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "as_pattern_binds_whole_and_subparts"
# subject = "match.as_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.as_pattern: an AS pattern binds the whole matched value while inner subpatterns bind too"""

# An AS pattern binds the whole matched value while subpatterns bind too.
match [1, 2]:
    case [a, b] as whole:
        pass
assert a == 1 and b == 2 and whole == [1, 2]
print("as_pattern_binds_whole_and_subparts OK")
