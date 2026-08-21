# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_lowercases_rest"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: capwords upper-cases the first letter and lower-cases the rest of each word: 'the quick BROWN fox' -> 'The Quick Brown Fox' and 'ABC DEF GHI' -> 'Abc Def Ghi'"""
import string

assert string.capwords("the quick BROWN fox") == "The Quick Brown Fox", "capwords lowercases rest"
assert string.capwords("ABC DEF GHI") == "Abc Def Ghi", "capwords all-upper input"
print("capwords_lowercases_rest OK")
