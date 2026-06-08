# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_decodes_fextra_header"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.GzipFile: a gzip member carrying an optional FEXTRA field (header flag 0x04) is decoded by skipping the extra field and still recovering the payload"""
import gzip
import io

# A gzip member may carry an optional FEXTRA field (flag bit 0x04 in the
# header). The decoder must skip it and still recover the payload. This
# fixed blob encodes b"Test" with a 5-byte extra field.
_with_extra = (
    b"\x1f\x8b\x08\x04\xb2\x17cQ\x02\xff\x05\x00Extra"
    b"\x0bI-.\x01\x002\xd1Mx\x04\x00\x00\x00"
)
with gzip.GzipFile(fileobj=io.BytesIO(_with_extra)) as _f:
    assert _f.read() == b"Test", "FEXTRA header decode"

print("gzipfile_decodes_fextra_header OK")
