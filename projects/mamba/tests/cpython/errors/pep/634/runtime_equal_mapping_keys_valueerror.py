# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "runtime_equal_mapping_keys_valueerror"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: two mapping-pattern keys that compare equal at runtime raise ValueError during matching"""

# Two mapping-pattern keys that compare equal at runtime raise ValueError.
class Keys:
    KEY = "a"


runtime_dup = False
try:
    match {"a": 0, "b": 1}:
        case {Keys.KEY: _y, "a": _z}:
            pass
except ValueError:
    runtime_dup = True
assert runtime_dup is True
print("runtime_equal_mapping_keys_valueerror OK")
