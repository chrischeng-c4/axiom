# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "name_is_int"
# subject = "token.NAME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.NAME: name_is_int (surface)."""
import token

assert type(token.NAME).__name__ == "int"
print("name_is_int OK")
