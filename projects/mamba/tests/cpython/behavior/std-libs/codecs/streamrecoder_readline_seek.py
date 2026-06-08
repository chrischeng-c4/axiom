# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "streamrecoder_readline_seek"
# subject = "codecs.EncodedFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.EncodedFile: a StreamRecoder (via EncodedFile over utf-16-le) supports readline and seek(0): two utf-16-le lines read back as b'line1\\n'/b'line2\\n' and seek(0) rewinds to the first"""
import codecs

import io
_bio = io.BytesIO("line1\nline2\n".encode("utf-16-le"))
_sr = codecs.EncodedFile(_bio, "utf-8", "utf-16-le")
assert _sr.readline() == b"line1\n"
_sr.seek(0)
assert _sr.readline() == b"line1\n", "seek(0) rewinds StreamRecoder"
assert _sr.readline() == b"line2\n"

print("streamrecoder_readline_seek OK")
