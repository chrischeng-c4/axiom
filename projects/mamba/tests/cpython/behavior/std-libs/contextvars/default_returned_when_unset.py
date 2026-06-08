# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "default_returned_when_unset"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a ContextVar declared with default= returns that default from get() while no value is set"""
import contextvars

cv = contextvars.ContextVar("with_default", default=42)
assert cv.get() == 42, f"default = {cv.get()!r}"
print("default_returned_when_unset OK")
