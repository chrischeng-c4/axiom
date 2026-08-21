# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "context_run_writes_do_not_leak"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: a set() performed inside Context.run() is visible there but does not leak back into the outer context after run() returns"""
import contextvars

cv = contextvars.ContextVar("leak", default="outer")
ctx = contextvars.copy_context()

def mutate():
    cv.set("inner")
    assert cv.get() == "inner", "mutation visible inside run"

ctx.run(mutate)
assert cv.get() == "outer", f"after run, outer unchanged = {cv.get()!r}"
print("context_run_writes_do_not_leak OK")
