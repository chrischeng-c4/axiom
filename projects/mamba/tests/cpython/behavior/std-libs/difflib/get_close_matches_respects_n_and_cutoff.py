# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_respects_n_and_cutoff"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches('helo', words, n=3, cutoff=0.5) returns at most 3 results and includes a real near-match like 'hello' or 'help'"""
import difflib

_words = ["help", "hello", "world", "helm", "hero"]
_m = difflib.get_close_matches("helo", _words, n=3, cutoff=0.5)
assert isinstance(_m, list), f"close_matches type = {type(_m)!r}"
assert len(_m) <= 3, f"at most n=3 results = {_m!r}"
assert "hello" in _m or "help" in _m, f"close match found: {_m!r}"
print("get_close_matches_respects_n_and_cutoff OK")
