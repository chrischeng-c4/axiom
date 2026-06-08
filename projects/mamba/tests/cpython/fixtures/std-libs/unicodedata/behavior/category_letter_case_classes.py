# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "category_letter_case_classes"
# subject = "unicodedata.category"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.category: category returns Lu/Ll/Lt for uppercase 'A', lowercase 'a', and titlecase digraph (U+01F2)"""
import unicodedata

assert unicodedata.category("A") == "Lu", "uppercase"
assert unicodedata.category("a") == "Ll", "lowercase"
assert unicodedata.category("ǲ") == "Lt", "titlecase Dz"  # U+01F2

print("category_letter_case_classes OK")
