# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "find_longest_match_popular_chars"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: with a 200+ length b that triggers the popular-element heuristic, find_longest_match still locates 'dabcd' -> Match(a=0,b=99,size=5)"""
import difflib

_a = "dabcd"
_b = "d" * 100 + "abc" + "d" * 100  # length over 200 so the popular heuristic applies
_sm = difflib.SequenceMatcher(a=_a, b=_b)
_match = _sm.find_longest_match(0, len(_a), 0, len(_b))
assert _match.a == 0, f"match.a = {_match.a!r}"
assert _match.b == 99, f"match.b = {_match.b!r}"
assert _match.size == 5, f"match.size = {_match.size!r}"
assert _a[_match.a:_match.a + _match.size] == _b[_match.b:_match.b + _match.size]
print("find_longest_match_popular_chars OK")
