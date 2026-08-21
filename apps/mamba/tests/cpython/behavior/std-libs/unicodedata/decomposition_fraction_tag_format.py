# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "decomposition_fraction_tag_format"
# subject = "unicodedata.decomposition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decomposition: the vulgar fraction one quarter (U+00BC) decomposes to the tagged form '<fraction> 0031 2044 0034'"""
import unicodedata

assert unicodedata.decomposition("¼") == "<fraction> 0031 2044 0034", (
    f"one-quarter decomposition = {unicodedata.decomposition(chr(0x00bc))!r}"
)

print("decomposition_fraction_tag_format OK")
