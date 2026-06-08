# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_single_metachar_substitutions"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate maps * -> .*, ? -> ., and pins the exact 3.12 strings for '*', '?', 'a?b*', '*.txt'"""
import fnmatch

assert fnmatch.translate("*") == "(?s:.*)\\Z", "star"
assert fnmatch.translate("?") == "(?s:.)\\Z", "question"
assert fnmatch.translate("a?b*") == "(?s:a.b.*)\\Z", "mixed metachars"
assert fnmatch.translate("*.txt") == "(?s:.*\\.txt)\\Z", "literal dot escaped"

print("translate_single_metachar_substitutions OK")
