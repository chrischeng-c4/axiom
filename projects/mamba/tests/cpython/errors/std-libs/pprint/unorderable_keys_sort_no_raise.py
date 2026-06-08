# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "unorderable_keys_sort_no_raise"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat({1:'a','two':'b'}, sort_dicts=True) does NOT raise: pprint catches the unorderable-key TypeError internally and falls back to insertion order"""
import pprint

# int vs str keys are unorderable, but pprint catches the TypeError from the
# attempted sort and falls back to insertion order instead of propagating it.
mixed = {1: "a", "two": "b"}
out = pprint.pformat(mixed, sort_dicts=True)
assert out == "{1: 'a', 'two': 'b'}", out
print("unorderable_keys_sort_no_raise OK")
