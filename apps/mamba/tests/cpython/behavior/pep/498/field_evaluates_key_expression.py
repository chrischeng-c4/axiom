# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_evaluates_key_expression"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: f-strings evaluate the index key expression while str.format treats [a] as the literal key 'a': with d={'a':'string',0:'integer'}, a=0, f'{d[a]}' is 'integer' but '{d[a]}'.format(d=d) is 'string'"""
# unlike str.format, an f-string index key is an evaluated expression

d = {"a": "string", 0: "integer"}
a = 0
assert f"{d[a]}" == "integer"             # a -> 0 -> "integer"
assert "{d[a]}".format(d=d) == "string"   # [a] -> literal key "a"

print("field_evaluates_key_expression OK")
