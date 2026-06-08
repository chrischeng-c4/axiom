# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "unidata_version_is_str"
# subject = "unicodedata.unidata_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.unidata_version: unidata_version_is_str (surface)."""
import unicodedata

assert type(unicodedata.unidata_version).__name__ == "str"
print("unidata_version_is_str OK")
