# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_find_and_count"
# subject = "str.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.find: find returns the first index or -1, count returns occurrences: 'hello world'.find('world')==6, .find('xyz')==-1, 'hello'.count('l')==2"""
import builtins  # noqa: F401

assert "hello world".find("world") == 6, "find hit index"
assert "hello world".find("xyz") == -1, "find miss is -1"
assert "hello".count("l") == 2, "count l"
assert "aaa".count("a") == 3, "count a"
print("str_find_and_count OK")
