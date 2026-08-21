# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "struct_class_is_callable"
# subject = "struct.Struct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: struct_class_is_callable (surface)."""
import struct

assert callable(struct.Struct)
print("struct_class_is_callable OK")
