# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "import_numbers"
# subject = "numbers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers: import_numbers (surface)."""
import numbers

assert hasattr(numbers, "Number")
print("import_numbers OK")
