# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "deflated_compresses_repetitive_data"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZIP_DEFLATED shrinks highly repetitive data so compress_size < file_size while the decompressed content round-trips exactly"""
import zipfile
import io

_data = b"aaaa" * 1000  # highly compressible
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_DEFLATED) as _zf:
    _zf.writestr("deflated.txt", _data)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _info = _zf.getinfo("deflated.txt")
    assert _info.compress_size < _info.file_size, "DEFLATED: compressed < original"
    assert _zf.read("deflated.txt") == _data, "DEFLATED: decompressed matches"

print("deflated_compresses_repetitive_data OK")
