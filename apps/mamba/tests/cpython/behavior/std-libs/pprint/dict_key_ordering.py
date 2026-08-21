# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "dict_key_ordering"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pprint sorts dict keys by default (and inside nested lists), sort_dicts=False preserves insertion order, and heterogeneous sortable keys order deterministically by key value"""
import pprint

# Default: keys are sorted alphabetically regardless of insertion order,
# including dicts nested inside a list.
d = {"b": 1, "a": 1, "c": 1}
assert pprint.pformat(d) == "{'a': 1, 'b': 1, 'c': 1}"
assert pprint.pformat([d, d]) == \
    "[{'a': 1, 'b': 1, 'c': 1}, {'a': 1, 'b': 1, 'c': 1}]"

# sort_dicts=False preserves insertion order.
ins = dict.fromkeys("cba")
assert pprint.pformat(ins, sort_dicts=False) == \
    "{'c': None, 'b': None, 'a': None}"
assert pprint.pformat([ins, ins], sort_dicts=False) == \
    "[{'c': None, 'b': None, 'a': None}, " \
    "{'c': None, 'b': None, 'a': None}]"

# Heterogeneous sortable keys order deterministically by pprint's safe order
# (int before str before tuple).
mixed = {"xy\tab\n": (3,), 5: [[]], (): {}}
assert pprint.pformat(mixed) == "{5: [[]], 'xy\\tab\\n': (3,), (): {}}"
print("dict_key_ordering OK")
