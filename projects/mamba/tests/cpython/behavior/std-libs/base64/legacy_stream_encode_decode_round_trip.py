# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "legacy_stream_encode_decode_round_trip"
# subject = "base64.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encode: the legacy base64.encode/base64.decode file-object API line-wraps to base64 and back through in-memory BytesIO streams"""
import base64
from io import BytesIO

# encode() reads raw bytes and writes line-wrapped base64 bytes.
_in = BytesIO(b"www.python.org")
_out = BytesIO()
base64.encode(_in, _out)
assert _out.getvalue() == b"d3d3LnB5dGhvbi5vcmc=\n", _out.getvalue()

# decode() reads base64 bytes and writes the decoded bytes back.
_in2 = BytesIO(_out.getvalue())
_out2 = BytesIO()
base64.decode(_in2, _out2)
assert _out2.getvalue() == b"www.python.org", _out2.getvalue()
print("legacy_stream_encode_decode_round_trip OK")
