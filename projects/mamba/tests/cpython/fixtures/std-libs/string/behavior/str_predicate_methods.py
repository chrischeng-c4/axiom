# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_predicate_methods"
# subject = "str.isdigit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.isdigit: the character-class predicates isdigit/isalpha/isalnum classify strings: '123'.isdigit() is True, 'abc 123'.isalnum() is False"""
import builtins  # noqa: F401

assert "123".isdigit() == True, "isdigit digits"
assert "abc".isdigit() == False, "isdigit letters"
assert "abc".isalpha() == True, "isalpha letters"
assert "123".isalpha() == False, "isalpha digits"
assert "abc123".isalnum() == True, "isalnum alphanumeric"
assert "abc 123".isalnum() == False, "isalnum with space"
print("str_predicate_methods OK")
