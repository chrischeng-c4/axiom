# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_abc_userdict_matched"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: a UserDict (mapping ABC) is matched, and **rest collects the other keys"""

import collections


# UserDict (mapping ABC) is also matched, and **rest collects the others.
ud = collections.UserDict({0: 1, 2: 3})
match ud:
    case {2: 3, **others}:
        pass
assert others == {0: 1}
print("mapping_abc_userdict_matched OK")
