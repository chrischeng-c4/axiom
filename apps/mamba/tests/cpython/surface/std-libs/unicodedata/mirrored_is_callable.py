# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "mirrored_is_callable"
# subject = "unicodedata.mirrored"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.mirrored: mirrored_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.mirrored)
print("mirrored_is_callable OK")
