# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "bom_utf16_le_attr"
# subject = "codecs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs: bom_utf16_le_attr (surface)."""
import codecs

assert hasattr(codecs, "BOM_UTF16_LE")
print("bom_utf16_le_attr OK")
