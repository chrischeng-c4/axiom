# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_finds_typo"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches('aple', ['apple', 'apricot', 'banana', 'mango']) ranks 'apple' first as the closest word"""
import difflib

_words = ["apple", "apricot", "banana", "mango"]
_matches = difflib.get_close_matches("aple", _words)
assert isinstance(_matches, list), f"close_matches type = {type(_matches)!r}"
assert _matches[0] == "apple", f"closest match ranked first = {_matches!r}"
print("get_close_matches_finds_typo OK")
