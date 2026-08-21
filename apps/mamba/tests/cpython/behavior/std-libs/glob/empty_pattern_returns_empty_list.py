# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "empty_pattern_returns_empty_list"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: glob('') returns [] (the empty pattern matches nothing)"""
import glob

assert glob.glob("") == [], "empty str pattern == []"
assert glob.glob(b"") == [], "empty bytes pattern == []"

print("empty_pattern_returns_empty_list OK")
