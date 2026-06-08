# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "prepare_class_is_callable"
# subject = "types.prepare_class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.prepare_class: prepare_class_is_callable (surface)."""
import types

assert callable(types.prepare_class)
print("prepare_class_is_callable OK")
