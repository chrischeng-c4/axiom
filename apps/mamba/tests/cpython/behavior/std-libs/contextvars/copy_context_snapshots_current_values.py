# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "copy_context_snapshots_current_values"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: a Context captured by copy_context sees the value present at copy time, not a later overwrite, when run() reads the var"""
import contextvars

cv = contextvars.ContextVar("snapshot")
cv.set("captured")
ctx = contextvars.copy_context()
cv.set("after_copy")
# Running inside the copied context sees the value at copy time, not the later one.
seen = []
ctx.run(lambda: seen.append(cv.get()))
assert seen == ["captured"], f"copy_context snapshot = {seen!r}"
print("copy_context_snapshots_current_values OK")
