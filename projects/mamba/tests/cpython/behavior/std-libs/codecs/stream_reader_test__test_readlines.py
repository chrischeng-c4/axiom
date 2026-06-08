# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "stream_reader_test__test_readlines"
# subject = "cpython.test_codecs.StreamReaderTest.test_readlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
import codecs
import contextlib
import copy
import io
import pickle
import sys
import encodings
self_reader = codecs.getreader('utf-8')
self_stream = io.BytesIO(b'\xed\x95\x9c\n\xea\xb8\x80')
f = self_reader(self_stream)
assert f.readlines() == ['한\n', '글']

print("StreamReaderTest::test_readlines: ok")
