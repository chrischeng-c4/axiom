# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "contextvar_name_attribute"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a freshly constructed ContextVar exposes its name via the read-only .name attribute"""
import contextvars

cv = contextvars.ContextVar("my_var")
assert cv.name == "my_var", f"name = {cv.name!r}"
print("contextvar_name_attribute OK")
