# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_case_methods"
# subject = "str.upper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.upper: case methods upper/lower/title/capitalize/swapcase produce the documented results, e.g. 'Hello World'.swapcase()=='hELLO wORLD'"""
import builtins  # noqa: F401

assert "hello".upper() == "HELLO", "upper"
assert "HELLO".lower() == "hello", "lower"
assert "hello world".title() == "Hello World", "title"
assert "hello world".capitalize() == "Hello world", "capitalize"
assert "Hello World".swapcase() == "hELLO wORLD", "swapcase"
print("str_case_methods OK")
