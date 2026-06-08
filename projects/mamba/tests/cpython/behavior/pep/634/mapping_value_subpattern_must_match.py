# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_value_subpattern_must_match"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a value subpattern in a mapping must also match the stored value or the case fails"""

# A value subpattern in a mapping must also match the stored value.
def has_yy(d):
    match d:
        case {"x": xv, "y": "yy", "z": zv}:
            return (xv, zv)
    return None


assert has_yy({"x": "x", "y": "yy", "z": "z"}) == ("x", "z")
assert has_yy({"x": "x", "y": "OTHER", "z": "z"}) is None
print("mapping_value_subpattern_must_match OK")
