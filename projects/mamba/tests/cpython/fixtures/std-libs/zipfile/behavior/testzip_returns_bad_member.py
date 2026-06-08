# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "testzip_returns_bad_member"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: testzip() returns the name of the first member whose stored CRC does not match its content"""
import zipfile
import io

# A crafted STORED entry "afile" whose recorded CRC is wrong.
_zip_bad_crc = (
    b"PK\x03\x04\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00\x00\x00"
    b"\x0c\x00\x00\x00\x05\x00\x00\x00afilehello,Aworld"
    b"PK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00"
    b"\x00\x00\x0c\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    b"\x80\x01\x00\x00\x00\x00afile"
    b"PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x003\x00\x00\x00/\x00\x00\x00\x00\x00"
)

with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    assert _zf.testzip() == "afile", f"testzip = {_zf.testzip()!r}"

print("testzip_returns_bad_member OK")
