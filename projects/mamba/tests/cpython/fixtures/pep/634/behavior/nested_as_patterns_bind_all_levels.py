# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "nested_as_patterns_bind_all_levels"
# subject = "match.as_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.as_pattern: nested as-patterns bind every intermediate value and the whole structure"""

# Nested as-patterns bind every intermediate value and the whole structure.
match ((0, 1), (2, 3)):
    case [(p as q, r) as left, (s, t) as right]:
        pass
assert p == 0 and q == 0 and r == 1 and left == (0, 1)
assert s == 2 and t == 3 and right == (2, 3)
print("nested_as_patterns_bind_all_levels OK")
