# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "import_warnings"
# subject = "warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: import_warnings (surface)."""
import warnings

assert hasattr(warnings, "warn")
print("import_warnings OK")
