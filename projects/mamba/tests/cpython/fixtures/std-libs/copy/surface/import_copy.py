# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "import_copy"
# subject = "copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""copy: import_copy (surface)."""
import copy

assert hasattr(copy, "copy")
print("import_copy OK")
