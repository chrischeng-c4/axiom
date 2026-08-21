# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_iso2022"
# dimension = "behavior"
# case = "iso2022_common_invalid_bytes_replace"
# subject = "cpython.test_codecencodings_iso2022.COMMON_CODEC_TESTS"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_iso2022.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""ISO-2022 codecs replace shared invalid byte sequences like CPython."""

CASES = [
    ("iso2022_jp", b"ab\xffcd", "replace", "ab\ufffdcd"),
    ("iso2022_jp", b"ab\x1bdef", "replace", "ab\x1bdef"),
    ("iso2022_jp", b"ab\x1b$def", "replace", "ab\ufffd"),
    ("iso2022_jp_2", b"ab\x1bNdef", "replace", "abdef"),
    ("iso2022_jp_3", b"ab\x1bNdef", "replace", "ab\x1bNdef"),
    ("iso2022_jp_2004", b"ab\x1bNdef", "replace", "ab\x1bNdef"),
    ("iso2022_kr", b"ab\x1bNdef", "replace", "ab\x1bNdef"),
]

for encoding, payload, errors, expected in CASES:
    actual = payload.decode(encoding, errors)
    assert actual == expected, (encoding, actual, expected)

print("iso2022_common_invalid_bytes_replace OK")
