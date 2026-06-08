# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "capture_binds_by_identity"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a capture binds to the same object (identity), not a copy"""

# A capture binds to the same object (identity), not a copy.
src = [1, 2, 3]
match {"data": src}:
    case {"data": captured}:
        pass
assert captured is src
print("capture_binds_by_identity OK")
