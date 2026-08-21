# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "surface"
# case = "import_inspect"
# subject = "inspect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: import_inspect (surface)."""
import inspect

assert hasattr(inspect, "currentframe")
print("import_inspect OK")
