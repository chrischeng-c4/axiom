# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "splitlines_unicode_line_boundaries"
# subject = "str.splitlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""str.splitlines: str.splitlines breaks on exactly the Unicode line-boundary set (LF/VT/FF/CR/FS/GS/RS/NEL/LS/PS) over the BMP, not just ASCII newlines"""
import unicodedata

# bug 7643: the code points str.splitlines treats as line boundaries.
BREAKERS = {
    0x0A,   # LINE FEED
    0x0B,   # LINE TABULATION
    0x0C,   # FORM FEED
    0x0D,   # CARRIAGE RETURN
    0x1C,   # FILE SEPARATOR
    0x1D,   # GROUP SEPARATOR
    0x1E,   # RECORD SEPARATOR
    0x85,   # NEXT LINE
    0x2028,  # LINE SEPARATOR
    0x2029,  # PARAGRAPH SEPARATOR
}
# A code point is a line boundary iff "<ch>A" splits into two pieces.
found = {i for i in range(0x10000) if len((chr(i) + "A").splitlines()) == 2}
assert found == BREAKERS, f"line-boundary set mismatch: {sorted(found ^ BREAKERS)!r}"

# Spot-check breakers and non-breakers explicitly.
assert "a\nb".splitlines() == ["a", "b"], "LF splits"
assert "a b".splitlines() == ["a", "b"], "LINE SEPARATOR splits"
assert "a b".splitlines() == ["a b"], "plain space does not split"
assert "a\tb".splitlines() == ["a\tb"], "tab does not split"

print("splitlines_unicode_line_boundaries OK")
