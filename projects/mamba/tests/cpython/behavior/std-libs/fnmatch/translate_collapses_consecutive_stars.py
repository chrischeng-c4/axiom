# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_collapses_consecutive_stars"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate squashes runs of consecutive stars to a single .*: '*********'->(?s:.*)\\Z, 'A*********'->(?s:A.*)\\Z, '*********A'->(?s:.*A)\\Z, 'A*********?[?]?'->(?s:A.*.[?].)\\Z"""
import fnmatch

assert fnmatch.translate("*********") == "(?s:.*)\\Z", "all stars collapse"
assert fnmatch.translate("A*********") == "(?s:A.*)\\Z", "leading literal"
assert fnmatch.translate("*********A") == "(?s:.*A)\\Z", "trailing literal"
assert fnmatch.translate("A*********?[?]?") == "(?s:A.*.[?].)\\Z", "stars + ? + class"

print("translate_collapses_consecutive_stars OK")
