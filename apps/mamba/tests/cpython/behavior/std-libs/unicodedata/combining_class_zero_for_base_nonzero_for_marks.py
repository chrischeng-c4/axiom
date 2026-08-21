# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "combining_class_zero_for_base_nonzero_for_marks"
# subject = "unicodedata.combining"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.combining: combining is 0 for base letters A/Z and positive for the combining grave (U+0300) and acute (U+0301) marks"""
import unicodedata

assert unicodedata.combining("A") == 0, "base letter A combining = 0"
assert unicodedata.combining("Z") == 0, "base letter Z combining = 0"
assert unicodedata.combining("̀") > 0, "combining grave > 0"
assert unicodedata.combining("́") > 0, "combining acute > 0"

print("combining_class_zero_for_base_nonzero_for_marks OK")
