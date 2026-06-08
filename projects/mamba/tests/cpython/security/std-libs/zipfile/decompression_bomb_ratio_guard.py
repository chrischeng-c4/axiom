# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "security"
# case = "decompression_bomb_ratio_guard"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: a DEFLATE bomb (48 MiB of zeros in a tiny archive) is contained by metadata before extraction: file_size/compress_size expose a >100x ratio up front, a size-cap guard refuses the member while admitting a benign small one, and a bounded streaming reader stops at its own 8 MiB limit without materialising the whole payload"""
import zipfile
import io

# Build the bomb in memory. 48 MiB of zeros deflate to a tiny entry.
PAYLOAD = b"\0" * (48 * 1024 * 1024)
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_DEFLATED) as _zf:
    _zf.writestr("bomb.bin", PAYLOAD)
_blob = _buf.getvalue()

# The compressed archive itself is tiny relative to the declared payload.
assert len(_blob) < 256 * 1024, f"archive too large: {len(_blob)}"

# --- Metadata exposes the ratio BEFORE extraction ---------------------------
with zipfile.ZipFile(io.BytesIO(_blob), "r") as _zf:
    _info = _zf.getinfo("bomb.bin")
    assert _info.compress_type == zipfile.ZIP_DEFLATED, _info.compress_type
    assert _info.file_size == len(PAYLOAD), f"file_size = {_info.file_size}"
    assert _info.compress_size < _info.file_size, "compress_size < file_size"
    assert _info.compress_size > 0, "compress_size > 0"
    _ratio = _info.file_size / _info.compress_size
    assert _ratio > 100.0, f"ratio = {_ratio}"

# --- A cap guard refuses to extract when file_size exceeds the cap ----------
CAP = 16 * 1024 * 1024  # Refuse anything claiming > 16 MiB uncompressed.


class BombRefused(Exception):
    pass


def _guarded_extract(archive_bytes, name, cap):
    with zipfile.ZipFile(io.BytesIO(archive_bytes), "r") as zf:
        zinfo = zf.getinfo(name)
        if zinfo.file_size > cap:
            raise BombRefused(f"{name}: declared {zinfo.file_size} > cap {cap}")
        return zf.read(name)


_refused = False
try:
    _guarded_extract(_blob, "bomb.bin", CAP)
except BombRefused:
    _refused = True
assert _refused, "guard must refuse bomb whose file_size exceeds cap"

# A modest entry under the cap is admitted by the same guard.
_small = io.BytesIO()
with zipfile.ZipFile(_small, "w", compression=zipfile.ZIP_DEFLATED) as _zf:
    _zf.writestr("ok.bin", b"\0" * (1024 * 1024))  # 1 MiB, under cap.
_data = _guarded_extract(_small.getvalue(), "ok.bin", CAP)
assert len(_data) == 1024 * 1024, f"admitted len = {len(_data)}"

# --- Bounded streaming reads the bomb without materializing it whole --------
LIMIT = 8 * 1024 * 1024  # Read at most 8 MiB then stop.
_read_total = 0
with zipfile.ZipFile(io.BytesIO(_blob), "r") as _zf:
    with _zf.open("bomb.bin", "r") as _fp:
        while _read_total < LIMIT:
            _piece = _fp.read(64 * 1024)
            if not _piece:
                break
            _read_total += len(_piece)
            assert _piece == b"\0" * len(_piece), "payload must be zeros"
assert _read_total == LIMIT, f"read_total = {_read_total}"

print("decompression_bomb_ratio_guard OK")
