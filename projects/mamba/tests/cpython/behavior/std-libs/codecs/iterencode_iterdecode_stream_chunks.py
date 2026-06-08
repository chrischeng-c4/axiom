# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "iterencode_iterdecode_stream_chunks"
# subject = "codecs.iterencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.iterencode: iterencode joins a chunked sequence into the full byte string and iterdecode is its inverse: ['foo','bar','café'] -> 'foobarcafé'.encode('utf-8') and back"""
import codecs

_joined = b"".join(codecs.iterencode(["foo", "bar", "café"], "utf-8"))
assert _joined == "foobarcafé".encode("utf-8"), f"iterencode = {_joined!r}"
_chunks = ["foo".encode("utf-8"), "café".encode("utf-8")]
_text = "".join(codecs.iterdecode(_chunks, "utf-8"))
assert _text == "foocafé", f"iterdecode = {_text!r}"

print("iterencode_iterdecode_stream_chunks OK")
