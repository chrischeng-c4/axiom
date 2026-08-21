# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "readbuffer_encode_accepts_buffer"
# subject = "codecs.readbuffer_encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.readbuffer_encode: readbuffer_encode accepts any buffer (e.g. a bytearray) and returns (bytes, length): readbuffer_encode(bytearray(b'spam')) is (b'spam', 4)"""
import codecs

assert codecs.readbuffer_encode(bytearray(b"spam")) == (b"spam", 4)

print("readbuffer_encode_accepts_buffer OK")
