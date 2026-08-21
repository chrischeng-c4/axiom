# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_basic"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: capwords title-cases each whitespace-delimited word: 'hello world foo' -> 'Hello World Foo'"""
import string

assert string.capwords("hello world foo") == "Hello World Foo", "capwords basic"
assert string.capwords("hello world") == "Hello World", "capwords two words"
assert string.capwords("abc def ghi") == "Abc Def Ghi", "capwords three words"
print("capwords_basic OK")
