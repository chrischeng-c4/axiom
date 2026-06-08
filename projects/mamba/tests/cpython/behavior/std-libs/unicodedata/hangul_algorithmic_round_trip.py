# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "hangul_algorithmic_round_trip"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: precomposed Hangul syllables decompose to six jamo under NFD and recompose losslessly under NFC (no UCD table entries)"""
import unicodedata

_hangul = "한글"  # two precomposed Hangul syllables
_hangul_nfd = unicodedata.normalize("NFD", _hangul)
assert len(_hangul_nfd) == 6, f"Hangul NFD len = {len(_hangul_nfd)!r}"
assert unicodedata.normalize("NFC", _hangul_nfd) == _hangul, "Hangul NFC round-trip"
assert unicodedata.normalize("NFC", _hangul) == _hangul, "already-composed Hangul stable"

print("hangul_algorithmic_round_trip OK")
