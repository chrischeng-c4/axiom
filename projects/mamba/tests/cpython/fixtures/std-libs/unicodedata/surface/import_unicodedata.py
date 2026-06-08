# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "import_unicodedata"
# subject = "unicodedata"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata: import_unicodedata (surface)."""
import unicodedata

assert hasattr(unicodedata, "name")
print("import_unicodedata OK")
