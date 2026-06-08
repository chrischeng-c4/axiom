# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "bom_utf8_value"
# subject = "codecs.BOM_UTF8"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.BOM_UTF8: bom_utf8_value (surface)."""
import codecs

assert type(codecs.BOM_UTF8).__name__ == "bytes"
print("bom_utf8_value OK")
