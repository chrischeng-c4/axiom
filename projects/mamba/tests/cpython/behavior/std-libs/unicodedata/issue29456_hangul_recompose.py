# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "issue29456_hangul_recompose"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFC of Hangul jamo sequences recomposes per issue #29456 (e.g. choseong+jungseong U+1100 U+1175 + jongseong collapses to the precomposed syllable)"""
import unicodedata

# issue #29456: Hangul jamo NFC recomposition corner cases.
# An L+V+T whose T composes stays as one syllable.
assert (unicodedata.normalize("NFC", "ᄀᅶᆨ")
        == "ᄀᅶᆨ"), "u1176 sequence stable"
# An L+V recomposes to a syllable; a trailing non-composing jongseong stays.
assert (unicodedata.normalize("NFC", "기ᆧ")
        == "기ᆧ"), "u11a7 L+V recomposes, T trails"
assert (unicodedata.normalize("NFC", "기ᇃ")
        == "기ᇃ"), "u11c3 L+V recomposes, T trails"

print("issue29456_hangul_recompose OK")
