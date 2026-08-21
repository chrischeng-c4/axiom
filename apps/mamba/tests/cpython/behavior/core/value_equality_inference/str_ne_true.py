# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "str_ne_true"
# subject = "str != str is True for differing values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""!= must be True for two strings whose content differs."""
a = "hello"
b = "world"
assert (a != b) is True, a != b
assert (a == b) is False, a == b
print("str_ne_true OK")
