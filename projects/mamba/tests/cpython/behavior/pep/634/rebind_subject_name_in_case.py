# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "rebind_subject_name_in_case"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: rebinding the same name as the subject (case x: from match x) is allowed and yields the value"""

# Rebinding the same name as the subject is allowed (case x: from match x).
x = 0
match x:
    case x:
        rebound = x
assert rebound == 0 and x == 0
print("rebind_subject_name_in_case OK")
