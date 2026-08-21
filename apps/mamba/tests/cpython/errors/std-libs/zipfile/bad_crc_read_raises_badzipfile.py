# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_crc_read_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: a crafted STORED entry whose recorded CRC does not match its content raises BadZipFile on read(), and also on a chunked open()/read() stream"""
import zipfile
import io

# A STORED entry "afile" whose recorded CRC does not match its content.
_zip_bad_crc = (
    b"PK\x03\x04\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00\x00\x00"
    b"\x0c\x00\x00\x00\x05\x00\x00\x00afilehello,Aworld"
    b"PK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00"
    b"\x00\x00\x0c\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    b"\x80\x01\x00\x00\x00\x00afile"
    b"PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x003\x00\x00\x00/\x00\x00\x00\x00\x00"
)

# read() of a bad-CRC member raises BadZipFile.
with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    _raised = False
    try:
        _zf.read("afile")
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "read of bad CRC -> BadZipFile"

# Streaming the member chunk by chunk also raises BadZipFile.
with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    _raised = False
    try:
        with _zf.open("afile", "r") as _fp:
            while _fp.read(2):
                pass
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "streamed read of bad CRC -> BadZipFile"

print("bad_crc_read_raises_badzipfile OK")
