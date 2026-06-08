# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_rest_captures_remaining_dict"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: **rest captures the remaining keys as a plain dict, and is the empty dict when all keys are consumed"""

# **rest captures the remaining keys as a plain dict.
match {"x": 1, "y": 2, "z": 3}:
    case {"x": x, **rest}:
        pass
assert x == 1
assert rest == {"y": 2, "z": 3}
assert type(rest) is dict

# **rest is the empty dict when all keys are consumed.
match {"only": 9}:
    case {"only": v, **leftover}:
        pass
assert v == 9 and leftover == {}
print("mapping_rest_captures_remaining_dict OK")
