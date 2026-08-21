# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "tok_name_maps_int_to_name"
# subject = "token.tok_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.tok_name: tok_name is a dict mapping each token-type int back to its symbolic name (ENDMARKER=0, NAME=1)"""
import token

# tok_name is a dict[int, str].
assert isinstance(token.tok_name, dict), type(token.tok_name).__name__

# It maps each token-type int back to that constant's symbolic name.
assert token.tok_name[0] == "ENDMARKER", token.tok_name[0]
assert token.tok_name[1] == "NAME", token.tok_name[1]
assert token.tok_name[token.NUMBER] == "NUMBER", token.tok_name[token.NUMBER]

# The round-trip closes: name -> value -> name for each public constant.
for name in ["ENDMARKER", "NAME", "NUMBER", "OP", "STRING"]:
    value = getattr(token, name)
    assert token.tok_name[value] == name, (name, value, token.tok_name[value])

print("tok_name_maps_int_to_name OK")
