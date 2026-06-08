# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_file_binary_roundtrip"
# subject = "tempfile.TemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryFile: TemporaryFile round-trips bytes: write then seek(0) then read returns the same bytes"""
import tempfile

_tempf = tempfile.TemporaryFile()
assert hasattr(_tempf, "read"), "TemporaryFile has read"
assert hasattr(_tempf, "write"), "TemporaryFile has write"
_tempf.write(b"test bytes")
_tempf.seek(0)
assert _tempf.read() == b"test bytes", "TemporaryFile round-trip"
_tempf.close()
print("temporary_file_binary_roundtrip OK")
