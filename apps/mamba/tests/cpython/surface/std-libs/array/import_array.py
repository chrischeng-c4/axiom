# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "import_array"
# subject = "array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array: import_array (surface)."""
import array

assert hasattr(array, "array")
print("import_array OK")
