# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "depth_truncation"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: unbounded depth reproduces repr; depth=1 collapses past the first level to (...)/{...}/[...]; depth=2 keeps two levels then truncates, across nested tuple/dict/list"""
import pprint

nested_tuple = (1, (2, (3, (4, (5, 6)))))
nested_dict = {1: {2: {3: {4: {5: {6: 6}}}}}}
nested_list = [1, [2, [3, [4, [5, [6, []]]]]]]

# Unbounded depth: pformat == repr for nested containers.
assert pprint.pformat(nested_tuple) == repr(nested_tuple)
assert pprint.pformat(nested_dict) == repr(nested_dict)
assert pprint.pformat(nested_list) == repr(nested_list)

# depth=1 collapses everything past the first level into the ellipsis form.
assert pprint.pformat(nested_tuple, depth=1) == "(1, (...))"
assert pprint.pformat(nested_dict, depth=1) == "{1: {...}}"
assert pprint.pformat(nested_list, depth=1) == "[1, [...]]"

# depth=2 keeps two levels, then truncates.
assert pprint.pformat(nested_tuple, depth=2) == "(1, (2, (...)))"
assert pprint.pformat(nested_list, depth=2) == "[1, [2, [...]]]"
print("depth_truncation OK")
