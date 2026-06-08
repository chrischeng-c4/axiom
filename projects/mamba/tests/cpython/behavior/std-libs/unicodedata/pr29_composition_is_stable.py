# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "pr29_composition_is_stable"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: PR-29 sequences are stable under NFC: normalize('NFC', text) returns text for the documented composition-exclusion cases"""
import unicodedata

# https://www.unicode.org/review/pr-29.html (issues #1054943, #10254):
# these sequences must be NFC-stable.
composed = (
    "େ̀ା",
    "ᄀ̀ᅡ",
    "Li̍t-sṳ́",
    "मार्क ज़"
    + "ुकेरबर्ग",
    "किर्गिज़"
    + "स्तान",
)
for text in composed:
    assert unicodedata.normalize("NFC", text) == text, f"PR-29 stable: {text!r}"

# issue #10254: a long run of C+combining-marks must not crash and is stable.
a = "C̸" * 20 + "Ç"
b = "C̸" * 20 + "\xC7"
assert unicodedata.normalize("NFC", a) == b, "issue10254 NFC"

print("pr29_composition_is_stable OK")
