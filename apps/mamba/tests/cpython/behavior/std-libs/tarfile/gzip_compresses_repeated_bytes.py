# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gzip_compresses_repeated_bytes"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a w:gz archive of 10000 identical bytes is smaller on the wire than the same payload written to a plain (uncompressed) w archive"""
import tarfile
import io

_data = b"x" * 10000  # compressible data

_buf_gz = io.BytesIO()
with tarfile.open(fileobj=_buf_gz, mode="w:gz") as _tf:
    _ti = tarfile.TarInfo("compressed.txt")
    _ti.size = len(_data)
    _tf.addfile(_ti, io.BytesIO(_data))
_size_gz = _buf_gz.tell()

_buf_plain = io.BytesIO()
with tarfile.open(fileobj=_buf_plain, mode="w") as _tf:
    _tip = tarfile.TarInfo("plain.txt")
    _tip.size = len(_data)
    _tf.addfile(_tip, io.BytesIO(_data))
_size_plain = _buf_plain.tell()

assert _size_gz < _size_plain, "gzip compresses repeated bytes"

print("gzip_compresses_repeated_bytes OK")
