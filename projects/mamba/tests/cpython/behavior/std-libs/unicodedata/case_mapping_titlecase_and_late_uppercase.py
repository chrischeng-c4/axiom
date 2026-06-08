# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "case_mapping_titlecase_and_late_uppercase"
# subject = "str.title"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""str.title: Unicode-aware case maps: DZ digraph (U+01C4/5/6) all titlecase to U+01C5; turned-g (U+1D79) uppercases to U+A77D; only U+0000 maps to a NUL"""
import unicodedata

# bug 4971: the DZ digraph titlecases to its single titlecase code point
# (U+01C5) from upper, title, and lower forms alike.
assert "Ǆ".title() == "ǅ", f"upper DZ title = {'Ǆ'.title()!r}"
assert "ǅ".title() == "ǅ", f"title DZ title = {'ǅ'.title()!r}"
assert "ǆ".title() == "ǅ", f"lower dz title = {'ǆ'.title()!r}"

# bug 1704793 / ucd_510: TURNED G (U+1D79) gained an uppercase (U+A77D)
# in a later Unicode version; lowercasing it is a no-op.
assert "ᵹ".lower() == "ᵹ", "turned g lower is identity"
assert "ᵹ".upper() == "Ᵹ", f"turned g upper = {'ᵹ'.upper()!r}"

# ASCII and punctuation behave as expected.
assert "a".upper() == "A", "ascii upper"
assert ".".upper() == ".", "punctuation upper is identity"

# bug 5828: across the whole code space, only U+0000 maps to a NUL.
_nul_producers = [c for c in range(0x110000)
                  if "\x00" in chr(c).lower() + chr(c).upper() + chr(c).title()]
assert _nul_producers == [0], f"NUL producers = {_nul_producers!r}"

print("case_mapping_titlecase_and_late_uppercase OK")
