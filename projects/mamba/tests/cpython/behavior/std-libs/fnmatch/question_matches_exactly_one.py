# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "question_matches_exactly_one"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: ? matches exactly one character: 'a?c' matches 'abc' but not 'ac' (too few) nor 'aXXc' (too many)"""
import fnmatch

assert fnmatch.fnmatchcase("abc", "a?c"), "? matches one"
assert not fnmatch.fnmatchcase("ac", "a?c"), "? requires exactly one"
assert not fnmatch.fnmatchcase("aXXc", "a?c"), "? does not match two"

print("question_matches_exactly_one OK")
