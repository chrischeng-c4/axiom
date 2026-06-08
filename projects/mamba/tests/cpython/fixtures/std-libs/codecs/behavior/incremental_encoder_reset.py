# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "incremental_encoder_reset"
# subject = "codecs.getincrementalencoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getincrementalencoder: an incremental utf-8 encoder encodes per call and reset() clears state: encode('ab') is b'ab', encode('é',True) is the utf-8 bytes, then reset() lets encode('xy') resume"""
import codecs

_enc = codecs.getincrementalencoder("utf-8")()
assert _enc.encode("ab") == b"ab"
assert _enc.encode("é", True) == "é".encode("utf-8")
_enc.reset()
assert _enc.encode("xy") == b"xy"

print("incremental_encoder_reset OK")
