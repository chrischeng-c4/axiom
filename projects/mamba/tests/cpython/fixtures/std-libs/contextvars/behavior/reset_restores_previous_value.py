# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "reset_restores_previous_value"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset(token) restores the value the ContextVar held before the matching set()"""
import contextvars

cv = contextvars.ContextVar("restore")
cv.set("first")
tok = cv.set("second")
assert cv.get() == "second", "value updated by the second set"
cv.reset(tok)
assert cv.get() == "first", "reset restores the value before the matching set"
print("reset_restores_previous_value OK")
