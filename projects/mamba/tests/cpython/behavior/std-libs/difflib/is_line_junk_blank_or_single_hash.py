# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "is_line_junk_blank_or_single_hash"
# subject = "difflib.IS_LINE_JUNK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.IS_LINE_JUNK: IS_LINE_JUNK is True for a line of only blanks and/or a single '#'; False for '##', non-blank text, etc."""
import difflib

for _line in ("#", "  ", " #", "# ", " # ", ""):
    assert difflib.IS_LINE_JUNK(_line), f"line junk true: {_line!r}"
for _line in ("##", " ##", "## ", "abc ", "abc #", "Mr. Moose is up!"):
    assert not difflib.IS_LINE_JUNK(_line), f"line junk false: {_line!r}"
print("is_line_junk_blank_or_single_hash OK")
