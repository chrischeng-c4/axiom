# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "walrus_in_guard_binds"
# subject = "match.guard_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.guard_pattern: a walrus assignment inside a guard binds and is visible in the case body"""

# A walrus assignment inside a guard binds and is visible in the case body.
match 7:
    case n if (doubled := n * 2):
        result = doubled
assert result == 14
print("walrus_in_guard_binds OK")
