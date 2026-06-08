# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfc_composes_to_single_char"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFC recomposes e-acute back into the single precomposed code point (length 1)"""
import unicodedata

_decomposed = "é"  # 'e' + combining acute
_precomposed = unicodedata.normalize("NFC", _decomposed)
assert _precomposed == "é", f"NFC compose = {_precomposed!r}"
assert len(_precomposed) == 1, f"NFC len = {len(_precomposed)!r}"

print("nfc_composes_to_single_char OK")
