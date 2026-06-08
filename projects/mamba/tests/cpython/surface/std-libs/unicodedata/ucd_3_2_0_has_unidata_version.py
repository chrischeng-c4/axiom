# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "ucd_3_2_0_has_unidata_version"
# subject = "unicodedata.ucd_3_2_0"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.ucd_3_2_0: ucd_3_2_0_has_unidata_version (surface)."""
import unicodedata

assert hasattr(unicodedata.ucd_3_2_0, "unidata_version")
print("ucd_3_2_0_has_unidata_version OK")
