# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "new_class_is_callable"
# subject = "types.new_class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.new_class: new_class_is_callable (surface)."""
import types

assert callable(types.new_class)
print("new_class_is_callable OK")
