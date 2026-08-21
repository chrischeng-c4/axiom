# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "insert_is_callable"
# subject = "array.array.insert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.insert: insert_is_callable (surface)."""
import array

assert callable(array.array.insert)
print("insert_is_callable OK")
