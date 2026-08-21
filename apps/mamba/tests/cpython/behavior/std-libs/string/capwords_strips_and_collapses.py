# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_strips_and_collapses"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: with the default separator capwords strips and collapses runs of whitespace: '   aBc  DeF   ' -> 'Abc Def' and tabs/newlines collapse to single spaces"""
import string

assert string.capwords("  hello  world  ") == "Hello World", "capwords strips/collapses spaces"
assert string.capwords("   aBc  DeF   ") == "Abc Def", "capwords strips edge runs"
assert string.capwords("abc\tdef\nghi") == "Abc Def Ghi", "capwords collapses tabs/newlines"
assert string.capwords("abc\t   def  \nghi") == "Abc Def Ghi", "capwords mixed whitespace"
print("capwords_strips_and_collapses OK")
