# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "builtin_single_capture_binds_subject"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a builtin class pattern with one capture (tuple(z)) binds the subject object itself"""

# A builtin class pattern with one capture binds the subject object itself.
empty = ()
match empty:
    case tuple(z):
        pass
assert z is empty
print("builtin_single_capture_binds_subject OK")
