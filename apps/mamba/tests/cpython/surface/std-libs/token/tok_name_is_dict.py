# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "tok_name_is_dict"
# subject = "token.tok_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.tok_name: tok_name_is_dict (surface)."""
import token

assert type(token.tok_name).__name__ == "dict"
print("tok_name_is_dict OK")
