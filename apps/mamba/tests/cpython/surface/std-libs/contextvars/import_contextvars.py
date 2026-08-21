# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "import_contextvars"
# subject = "contextvars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars: import_contextvars (surface)."""
import contextvars

assert hasattr(contextvars, "ContextVar")
print("import_contextvars OK")
