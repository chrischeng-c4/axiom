# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "isnonterminal_is_callable"
# subject = "token.ISNONTERMINAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.ISNONTERMINAL: isnonterminal_is_callable (surface)."""
import token

assert callable(token.ISNONTERMINAL)
print("isnonterminal_is_callable OK")
