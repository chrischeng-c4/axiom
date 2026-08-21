# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfkd_expands_compatibility_ligature"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFKD maps the fi ligature (U+FB01) to the ASCII pair 'fi'"""
import unicodedata

_nfkd = unicodedata.normalize("NFKD", "ﬁ")  # fi ligature U+FB01
assert _nfkd == "fi", f"NFKD ligature = {_nfkd!r}"

print("nfkd_expands_compatibility_ligature OK")
