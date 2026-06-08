# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "set_returns_token_and_updates_value"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: set(v) returns a contextvars.Token and get() afterwards returns v"""
import contextvars

cv = contextvars.ContextVar("setget")
tok = cv.set("hello")
assert isinstance(tok, contextvars.Token), f"set() returns a Token, got {type(tok)!r}"
assert cv.get() == "hello", f"after set = {cv.get()!r}"
print("set_returns_token_and_updates_value OK")
