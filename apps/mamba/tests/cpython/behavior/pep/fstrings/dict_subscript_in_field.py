# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dict_subscript_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a dict subscript evaluates inside a field: with d={'key':'value'}, f'{d["key"]}' is 'value'"""
# an f-string index key is an evaluated subscript expression

d = {"key": "value"}
assert f"{d['key']}" == "value", "dict value in f-string"

print("dict_subscript_in_field OK")
