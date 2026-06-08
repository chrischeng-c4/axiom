# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_replace"
# subject = "str.replace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.replace: str.replace substitutes every occurrence of the substring: 'hello world'.replace('world','mamba')=='hello mamba', 'aaa'.replace('a','b')=='bbb'"""
import builtins  # noqa: F401

assert "hello world".replace("world", "mamba") == "hello mamba", "replace word"
assert "aaa".replace("a", "b") == "bbb", "replace all chars"
print("str_replace OK")
