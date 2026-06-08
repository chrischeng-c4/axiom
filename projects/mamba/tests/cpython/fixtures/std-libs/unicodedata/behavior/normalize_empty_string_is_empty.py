# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "normalize_empty_string_is_empty"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: normalizing the empty string yields the empty string for every form NFC/NFD/NFKC/NFKD"""
import unicodedata

for _form in ("NFC", "NFD", "NFKC", "NFKD"):
    assert unicodedata.normalize(_form, "") == "", f"empty {_form}"

print("normalize_empty_string_is_empty OK")
