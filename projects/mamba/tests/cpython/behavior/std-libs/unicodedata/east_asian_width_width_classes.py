# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "east_asian_width_width_classes"
# subject = "unicodedata.east_asian_width"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.east_asian_width: east_asian_width returns Na for space, W for a CJK char (U+C894), H for halfwidth (U+FF66), F for fullwidth (U+FF1F)"""
import unicodedata

assert unicodedata.east_asian_width("\x20") == "Na", "space is narrow"
assert unicodedata.east_asian_width("좔") == "W", "CJK is wide"
assert unicodedata.east_asian_width("ｦ") == "H", "halfwidth katakana is H"
assert unicodedata.east_asian_width("？") == "F", "fullwidth question mark is F"
assert unicodedata.east_asian_width("‐") == "A", "hyphen is ambiguous"

print("east_asian_width_width_classes OK")
