# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "import_binascii"
# subject = "binascii"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii: import_binascii (surface)."""
import binascii

assert hasattr(binascii, "hexlify")
print("import_binascii OK")
