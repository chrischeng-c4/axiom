# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "import_struct"
# subject = "struct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct: import_struct (surface)."""
import struct

assert hasattr(struct, "pack")
print("import_struct OK")
