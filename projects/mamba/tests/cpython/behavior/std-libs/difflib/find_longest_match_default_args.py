# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "find_longest_match_default_args"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: find_longest_match() with no args finds the largest block of 'foo bar' in 'foo baz bar' -> Match(a=0,b=0,size=6)"""
import difflib

_a = "foo bar"
_b = "foo baz bar"
_sm = difflib.SequenceMatcher(a=_a, b=_b)
_match = _sm.find_longest_match()
assert _match.a == 0, f"match.a = {_match.a!r}"
assert _match.b == 0, f"match.b = {_match.b!r}"
assert _match.size == 6, f"match.size = {_match.size!r}"
# The matched spans must be identical in both sequences.
assert _a[_match.a:_match.a + _match.size] == _b[_match.b:_match.b + _match.size]
print("find_longest_match_default_args OK")
