# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "punycode_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: punycode bootstring round-trips a non-ASCII string ('他们为什么不说中文' -> b'ihqwcrb4cv8a8dqg056pqjye') and appends only '-' for pure ASCII ('abc' -> b'abc-')"""
import codecs

_uni = "他们为什么不说中文"
_puny = _uni.encode("punycode")
assert _puny == b"ihqwcrb4cv8a8dqg056pqjye", f"punycode encode = {_puny!r}"
assert _puny.decode("punycode") == _uni, "punycode round-trip"
# Pure-ASCII just appends the '-' delimiter.
assert "abc".encode("punycode") == b"abc-"

print("punycode_roundtrip OK")
