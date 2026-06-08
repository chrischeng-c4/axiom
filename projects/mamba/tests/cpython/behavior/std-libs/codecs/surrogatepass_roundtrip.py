# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "surrogatepass_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogatepass lets a lone surrogate pass through utf-8: '\\ud901'.encode('utf-8','surrogatepass') is b'\\xed\\xa4\\x81' and decodes back to '\\ud901'"""
import codecs

_data = "\ud901".encode("utf-8", "surrogatepass")
assert _data == b"\xed\xa4\x81", f"surrogatepass utf-8 = {_data!r}"
assert _data.decode("utf-8", "surrogatepass") == "\ud901"

print("surrogatepass_roundtrip OK")
