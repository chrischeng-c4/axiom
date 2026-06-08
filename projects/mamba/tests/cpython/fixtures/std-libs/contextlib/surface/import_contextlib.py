# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "import_contextlib"
# subject = "contextlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib: import_contextlib (surface)."""
import contextlib

assert hasattr(contextlib, "contextmanager")
print("import_contextlib OK")
