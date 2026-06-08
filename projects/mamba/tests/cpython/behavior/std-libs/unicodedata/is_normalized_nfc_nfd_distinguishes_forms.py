# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "is_normalized_nfc_nfd_distinguishes_forms"
# subject = "unicodedata.is_normalized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.is_normalized: is_normalized reports True/False correctly for precomposed vs decomposed e-acute under NFC and NFD"""
import unicodedata

_precomp = "é"     # precomposed e-acute (U+00E9)
_decomp = "é"     # decomposed: 'e' + combining acute (U+0301)
assert unicodedata.is_normalized("NFC", _precomp) is True, "precomposed is NFC"
assert unicodedata.is_normalized("NFD", _precomp) is False, "precomposed not NFD"
assert unicodedata.is_normalized("NFD", _decomp) is True, "decomposed is NFD"
assert unicodedata.is_normalized("NFC", _decomp) is False, "decomposed not NFC"

print("is_normalized_nfc_nfd_distinguishes_forms OK")
