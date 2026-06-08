# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "ucd_3_2_0_pins_old_unicode_version"
# subject = "unicodedata.ucd_3_2_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.ucd_3_2_0: the frozen ucd_3_2_0 database reports unidata_version '3.2.0' and disagrees with the live UCD on a post-3.2 mirrored change (U+0F3A) while stable properties agree"""
import unicodedata

ucd_old = unicodedata.ucd_3_2_0

# The pinned database carries its own (older) version string.
assert ucd_old.unidata_version == "3.2.0", f"pinned version = {ucd_old.unidata_version!r}"
assert unicodedata.unidata_version != "3.2.0", "live UCD is newer than 3.2.0"

# bug ucd_510: U+0F3A became mirrored after Unicode 3.2, so the two
# databases disagree on .mirrored().
_ch = "༺"  # TIBETAN MARK GTER YIG MGO UM RNAM BCAD MA
assert unicodedata.mirrored(_ch) == 1, f"live mirrored = {unicodedata.mirrored(_ch)!r}"
assert ucd_old.mirrored(_ch) == 0, f"3.2.0 mirrored = {ucd_old.mirrored(_ch)!r}"

# Properties that predate Unicode 3.2 match in both databases.
for _q in (lambda u: u.name("A"),
           lambda u: u.category("A"),
           lambda u: u.bidirectional("A"),
           lambda u: u.combining("A")):
    assert _q(unicodedata) == _q(ucd_old), "stable property agrees across UCD versions"

print("ucd_3_2_0_pins_old_unicode_version OK")
