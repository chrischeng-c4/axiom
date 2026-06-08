# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "exact_token_types_is_dict"
# subject = "token.EXACT_TOKEN_TYPES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.EXACT_TOKEN_TYPES: exact_token_types_is_dict (surface)."""
import token

assert type(token.EXACT_TOKEN_TYPES).__name__ == "dict"
print("exact_token_types_is_dict OK")
