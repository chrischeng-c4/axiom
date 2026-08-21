# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "reset_to_unset_after_first_set"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset(token) of the first-ever set on a no-default var returns it to the unset state, so get() raises LookupError again"""
import contextvars

cv = contextvars.ContextVar("unset_after_reset")
tok = cv.set("only")
assert cv.get() == "only", "value visible while set"
cv.reset(tok)
_raised = False
try:
    cv.get()
except LookupError:
    _raised = True
assert _raised, "after resetting the first set, the var is unset again -> LookupError"
print("reset_to_unset_after_first_set OK")
