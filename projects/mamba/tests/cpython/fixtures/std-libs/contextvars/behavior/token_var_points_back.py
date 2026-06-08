# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_var_points_back"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: the Token returned by set() carries .var identical (is) to the ContextVar it came from"""
import contextvars

cv = contextvars.ContextVar("tok_var")
tok = cv.set("first")
assert tok.var is cv, "Token.var is the originating ContextVar"
print("token_var_points_back OK")
