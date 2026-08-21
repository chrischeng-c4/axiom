# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "iseof_is_callable"
# subject = "token.ISEOF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.ISEOF: iseof_is_callable (surface)."""
import token

assert callable(token.ISEOF)
print("iseof_is_callable OK")
