# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "decomposition_string_for_precomposed"
# subject = "unicodedata.decomposition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decomposition: decomposition('A') is empty while e-acute yields a non-empty mapping starting with code point 0065"""
import unicodedata

assert unicodedata.decomposition("A") == "", f"A decomposition = {unicodedata.decomposition('A')!r}"
_d = unicodedata.decomposition("é")  # precomposed e-acute
assert _d != "", f"e-acute has decomposition = {_d!r}"
assert _d.startswith("0065"), f"decomposition starts with base 'e' = {_d!r}"

print("decomposition_string_for_precomposed OK")
