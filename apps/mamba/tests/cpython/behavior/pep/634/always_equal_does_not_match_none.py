# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "always_equal_does_not_match_none"
# subject = "match.singleton_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.singleton_pattern: an __eq__ that always returns True does not let a value match the None singleton pattern"""

# An __eq__ that always returns True does not let a value match the None pattern.
class AlwaysEqual:
    def __eq__(self, other):
        return True


probe = AlwaysEqual()
matched_none = False
match probe:
    case None:
        matched_none = True
assert matched_none is False
print("always_equal_does_not_match_none OK")
