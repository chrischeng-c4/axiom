# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_old_value_missing_when_unset"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: the Token from the first set() of a previously-unset var has old_value identical to contextvars.Token.MISSING"""
import contextvars

cv = contextvars.ContextVar("tok_missing")
tok = cv.set("first")
assert tok.old_value is contextvars.Token.MISSING, "old_value is MISSING when the var was previously unset"
print("token_old_value_missing_when_unset OK")
