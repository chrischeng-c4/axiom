# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "exact_token_types_maps_op_to_int"
# subject = "token.EXACT_TOKEN_TYPES"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.EXACT_TOKEN_TYPES: EXACT_TOKEN_TYPES maps an operator string to its token-type int ('+' -> PLUS=14) and omits unknown operators"""
import token

# EXACT_TOKEN_TYPES is a dict[str, int] keyed by the operator's literal text.
assert isinstance(token.EXACT_TOKEN_TYPES, dict), type(token.EXACT_TOKEN_TYPES).__name__

# '+' resolves to PLUS (14); the mapped value equals the named constant.
assert token.EXACT_TOKEN_TYPES["+"] == 14, token.EXACT_TOKEN_TYPES["+"]
assert token.EXACT_TOKEN_TYPES["+"] == token.PLUS, token.EXACT_TOKEN_TYPES["+"]
assert token.EXACT_TOKEN_TYPES["=="] == token.EQEQUAL, token.EXACT_TOKEN_TYPES["=="]

# Unknown operator strings are simply absent (no entry, not a sentinel).
assert "ZZ" not in token.EXACT_TOKEN_TYPES
assert "NAME" not in token.EXACT_TOKEN_TYPES

print("exact_token_types_maps_op_to_int OK")
