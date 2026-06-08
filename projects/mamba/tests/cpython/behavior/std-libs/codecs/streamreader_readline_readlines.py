# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "streamreader_readline_readlines"
# subject = "codecs.getreader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getreader: a StreamReader splits on newlines and decodes each line: readline over '한\\n글' keeps the newline then returns the unterminated line then '' at EOF; readlines returns both lines"""
import codecs

import io
# readline keeps the newline; the last line is unterminated; EOF -> ''.
_stream = io.BytesIO("한\n글".encode("utf-8"))
_reader = codecs.getreader("utf-8")(_stream)
assert _reader.readline() == "한\n", "readline keeps the newline"
assert _reader.readline() == "글", "last (unterminated) line"
assert _reader.readline() == "", "EOF -> empty string"
# readlines returns every line.
_stream2 = io.BytesIO("한\n글".encode("utf-8"))
assert codecs.getreader("utf-8")(_stream2).readlines() == ["한\n", "글"]

print("streamreader_readline_readlines OK")
