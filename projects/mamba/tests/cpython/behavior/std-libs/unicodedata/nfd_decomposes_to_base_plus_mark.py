# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfd_decomposes_to_base_plus_mark"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFD splits precomposed e-acute into base 'e' (U+0065) followed by combining acute (U+0301)"""
import unicodedata

_nfd = unicodedata.normalize("NFD", "é")  # precomposed e-acute
assert len(_nfd) == 2, f"NFD length = {len(_nfd)!r}"
assert ord(_nfd[0]) == 0x65, f"NFD base = {ord(_nfd[0]):#x}"
assert ord(_nfd[1]) == 0x301, f"NFD mark = {ord(_nfd[1]):#x}"

print("nfd_decomposes_to_base_plus_mark OK")
