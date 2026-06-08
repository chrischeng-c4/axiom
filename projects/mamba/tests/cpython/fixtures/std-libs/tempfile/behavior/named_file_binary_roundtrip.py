# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_binary_roundtrip"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: bytes written to a NamedTemporaryFile(delete=False) survive a close and reopen-by-name read"""
import os
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _ntf:
    _ntf_name = _ntf.name
    _ntf.write(b"\x00\x01\x02\x03")
    _ntf.flush()
try:
    with open(_ntf_name, "rb") as _f:
        _data = _f.read()
    assert _data == b"\x00\x01\x02\x03", f"ntf binary = {_data!r}"
finally:
    os.unlink(_ntf_name)
print("named_file_binary_roundtrip OK")
