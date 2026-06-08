# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "import_traceback"
# subject = "traceback"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback: import_traceback (surface)."""
import traceback

assert hasattr(traceback, "format_exc")
print("import_traceback OK")
