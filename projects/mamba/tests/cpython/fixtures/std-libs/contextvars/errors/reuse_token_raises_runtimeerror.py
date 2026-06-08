# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "reuse_token_raises_runtimeerror"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: resetting with a Token a second time (the Token was already consumed) raises RuntimeError"""
import contextvars

cv = contextvars.ContextVar("reuse")
cv.set(1)
tok = cv.set(2)
cv.reset(tok)  # first reset consumes the token
_raised = False
try:
    cv.reset(tok)  # second reset of the same, now-used, token
except RuntimeError:
    _raised = True
assert _raised, "reusing a consumed Token must raise RuntimeError"
print("reuse_token_raises_runtimeerror OK")
