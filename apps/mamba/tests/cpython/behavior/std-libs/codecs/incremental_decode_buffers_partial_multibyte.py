# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "incremental_decode_buffers_partial_multibyte"
# subject = "codecs.getincrementaldecoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getincrementaldecoder: an incremental utf-8 decoder buffers a multi-byte char split across single-byte calls and yields it only once complete; final=True flushes"""
import codecs

# A multi-byte char split across single-byte calls buffers until complete.
_dec = codecs.getincrementaldecoder("utf-8")()
_full = "café 日本".encode("utf-8")
_out = ""
for _i in range(len(_full)):
    _out += _dec.decode(_full[_i : _i + 1])
_out += _dec.decode(b"", True)
assert _out == "café 日本", f"incremental utf-8 decode = {_out!r}"
# Splitting a 2-byte char yields '' until the second byte arrives.
_dec2 = codecs.getincrementaldecoder("utf-8")()
_two = "é".encode("utf-8")  # 0xc3 0xa9
assert _dec2.decode(_two[:1]) == "", "partial multibyte -> empty"
assert _dec2.decode(_two[1:], True) == "é", "completion yields the char"

print("incremental_decode_buffers_partial_multibyte OK")
