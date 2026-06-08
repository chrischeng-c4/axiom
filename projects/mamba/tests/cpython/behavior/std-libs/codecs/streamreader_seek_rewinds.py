# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "streamreader_seek_rewinds"
# subject = "codecs.getreader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getreader: seek(0) on a StreamReader rewinds it so a second read() repeats the full content"""
import codecs

import io
_payload = ("abc123" * 10 + "\n") * 2
_stream = io.BytesIO(_payload.encode("utf-8"))
_reader = codecs.getreader("utf-8")(_stream)
assert _reader.read() == _payload
_reader.seek(0, 0)
assert _reader.read() == _payload, "read after seek(0) repeats"

print("streamreader_seek_rewinds OK")
