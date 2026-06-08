# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "thread_starts_with_own_context"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a new thread starts with its own context: it sees the var's default (not the main thread's set value), and its own writes do not leak back to main"""
import contextvars
import threading

cv = contextvars.ContextVar("threaded", default="default")
cv.set("main_value")

saw = []

def thread_fn():
    saw.append(cv.get())  # a fresh thread starts from the default, not main's value
    cv.set("thread_val")
    saw.append(cv.get())

t = threading.Thread(target=thread_fn)
t.start()
t.join()

assert saw[0] == "default", f"thread starts at default = {saw[0]!r}"
assert saw[1] == "thread_val", f"thread sees its own write = {saw[1]!r}"
assert cv.get() == "main_value", f"main value unchanged by the thread = {cv.get()!r}"
print("thread_starts_with_own_context OK")
