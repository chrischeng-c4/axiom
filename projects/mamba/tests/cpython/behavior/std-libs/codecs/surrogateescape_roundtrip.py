# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "surrogateescape_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogateescape maps undecodable bytes 0x80-0xFF to lone surrogates U+DC80-U+DCFF and back, for both ascii and utf-8, including an ill-formed 3-byte sequence"""
import codecs

# ascii smuggles a high byte through a lone surrogate.
assert b"foo\x80bar".decode("ascii", "surrogateescape") == "foo\udc80bar"
assert "foo\udc80bar".encode("ascii", "surrogateescape") == b"foo\x80bar"
# utf-8 round-trips the same way.
assert b"foo\x80bar".decode("utf-8", "surrogateescape") == "foo\udc80bar"
assert "foo\udc80bar".encode("utf-8", "surrogateescape") == b"foo\x80bar"
# An ill-formed 3-byte UTF-8 surrogate becomes three escapes.
assert b"\xed\xb0\x80".decode("utf-8", "surrogateescape") == "\udced\udcb0\udc80"
assert "\udced\udcb0\udc80".encode("utf-8", "surrogateescape") == b"\xed\xb0\x80"

print("surrogateescape_roundtrip OK")
