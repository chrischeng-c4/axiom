# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "incomplete_attr_exists"
# subject = "binascii"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii: incomplete_attr_exists (surface)."""
import binascii

assert hasattr(binascii, "Incomplete")
print("incomplete_attr_exists OK")
