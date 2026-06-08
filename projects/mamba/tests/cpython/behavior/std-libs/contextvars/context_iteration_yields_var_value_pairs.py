# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "context_iteration_yields_var_value_pairs"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: iterating a Context yields its ContextVar keys; dict(context) maps each captured var to its value, and membership-tests by var"""
import contextvars

cv_a = contextvars.ContextVar("iter_a")
cv_b = contextvars.ContextVar("iter_b")
cv_a.set("val_a")
cv_b.set("val_b")
ctx = contextvars.copy_context()
items = dict(ctx)
assert cv_a in items, "cv_a present as a key in the context"
assert items[cv_a] == "val_a", f"cv_a value = {items[cv_a]!r}"
assert items[cv_b] == "val_b", f"cv_b value = {items[cv_b]!r}"
print("context_iteration_yields_var_value_pairs OK")
