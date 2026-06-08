# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "isterminal_is_callable"
# subject = "token.ISTERMINAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token.ISTERMINAL: isterminal_is_callable (surface)."""
import token

assert callable(token.ISTERMINAL)
print("isterminal_is_callable OK")
