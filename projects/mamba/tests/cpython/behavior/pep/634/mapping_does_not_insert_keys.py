# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_does_not_insert_keys"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: matching a defaultdict does not auto-create missing keys; the subject is unchanged"""

import collections


# defaultdict is matched as-is; matching does NOT auto-create missing keys.
dd = collections.defaultdict(int)
match dd:
    case {0: 0}:
        which = "had-zero"
    case {**everything}:
        which = "empty"
assert which == "empty"
assert dd == {}  # matching {0: 0} did not insert key 0
print("mapping_does_not_insert_keys OK")
