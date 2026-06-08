# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "is_character_junk_space_tab_only"
# subject = "difflib.IS_CHARACTER_JUNK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.IS_CHARACTER_JUNK: IS_CHARACTER_JUNK is True only for ' ' and '\\t'; False for 'a', '#', newline, formfeed, carriage-return, vertical-tab"""
import difflib

for _ch in (" ", "\t"):
    assert difflib.IS_CHARACTER_JUNK(_ch), f"char junk true: {_ch!r}"
for _ch in ("a", "#", "\n", "\x0c", "\r", "\x0b"):
    assert not difflib.IS_CHARACTER_JUNK(_ch), f"char junk false: {_ch!r}"
print("is_character_junk_space_tab_only OK")
