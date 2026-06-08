# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "stn_nts_pad_and_truncate"
# subject = "tarfile.stn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.stn: stn nul-pads a string into a fixed-width field and truncates an overlong one; nts decodes a nul-terminated field back, stopping at the first nul"""
import tarfile

# stn: encode a string into a fixed-width nul-padded field (truncating).
assert tarfile.stn("foo", 8, "ascii", "strict") == b"foo\x00\x00\x00\x00\x00", "stn pad"
assert tarfile.stn("foobar", 3, "ascii", "strict") == b"foo", "stn truncate"

# nts: decode a nul-terminated field back to a string (stops at first nul).
assert tarfile.nts(b"foo\x00\x00\x00\x00\x00", "ascii", "strict") == "foo", "nts pad"
assert tarfile.nts(b"foo\x00bar\x00", "ascii", "strict") == "foo", "nts stops at nul"

print("stn_nts_pad_and_truncate OK")
