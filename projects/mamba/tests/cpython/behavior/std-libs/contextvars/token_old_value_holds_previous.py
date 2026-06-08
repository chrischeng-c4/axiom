# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_old_value_holds_previous"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: overwriting an already-set var yields a Token whose old_value is the prior value"""
import contextvars

cv = contextvars.ContextVar("tok_prev")
cv.set("first")
tok = cv.set("second")
assert tok.old_value == "first", f"old_value = {tok.old_value!r}"
print("token_old_value_holds_previous OK")
