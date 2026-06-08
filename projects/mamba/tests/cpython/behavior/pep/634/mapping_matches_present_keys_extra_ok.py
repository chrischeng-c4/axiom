# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_matches_present_keys_extra_ok"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a mapping pattern matches when the named keys are present; extra keys are ignored, missing keys fail"""

# A mapping pattern matches when the named keys are present; extra keys are OK.
def route(cfg):
    match cfg:
        case {"bandwidth": b, "latency": l}:
            return (b, l)
    return None


assert route({"bandwidth": 0, "latency": 1}) == (0, 1)
assert route({"bandwidth": 0, "latency": 1, "extra": 2}) == (0, 1)  # extra ignored
assert route({"bandwidth": 0}) is None  # missing required key
print("mapping_matches_present_keys_extra_ok OK")
