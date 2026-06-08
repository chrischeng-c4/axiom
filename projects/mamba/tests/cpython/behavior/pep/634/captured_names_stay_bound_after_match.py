# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "captured_names_stay_bound_after_match"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: names captured by the matched case remain bound after the match statement"""

# Names captured by the matched case stay bound after the match statement.
match (10, 20):
    case (a, b):
        pass
assert a == 10 and b == 20
print("captured_names_stay_bound_after_match OK")
