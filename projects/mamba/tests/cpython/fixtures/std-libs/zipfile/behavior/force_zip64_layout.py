# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "force_zip64_layout"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: open(..., force_zip64=True) writes the ZIP64 layout even for a 1-byte payload: the fixed header sizes are the 0xFFFFFFFF sentinel, a Zip64 extra field (id 1) carries the true 8-byte sizes, and the reopened member reports extract_version >= ZIP64_VERSION"""
import zipfile
import io
import struct

_data = io.BytesIO()
with zipfile.ZipFile(_data, mode="w", allowZip64=True) as _zf:
    with _zf.open("text.txt", mode="w", force_zip64=True) as _member:
        _member.write(b"_")
_zipdata = _data.getvalue()

(_header, _vers, _os_byte, _flags, _comp, _csize, _usize, _fn_len, _ex_total_len,
 _filename, _ex_id, _ex_len, _ex_usize, _ex_csize, _cd_sig) = struct.unpack(
    "<4sBBHH8xIIHH8shhQQx4s", _zipdata[:63])

assert _header == b"PK\x03\x04", f"local file header sig = {_header!r}"
assert _vers >= zipfile.ZIP64_VERSION, f"version needed = {_vers!r}"
assert _os_byte == 0, f"os byte = {_os_byte!r}"
assert _flags == 0, f"flags = {_flags!r}"
assert _comp == 0, f"compression = {_comp!r}"
# Sizes in the fixed header are the 0xFFFFFFFF sentinel under ZIP64.
assert _csize == 4294967295, f"header compress_size sentinel = {_csize!r}"
assert _usize == 4294967295, f"header file_size sentinel = {_usize!r}"
assert _fn_len == 8, f"filename length = {_fn_len!r}"
assert _filename == b"text.txt", f"filename = {_filename!r}"
# The Zip64 extra field carries the true sizes.
assert _ex_total_len == 20, f"extra total length = {_ex_total_len!r}"
assert _ex_id == 1, f"extra id = {_ex_id!r}"
assert _ex_len == 16, f"extra payload length = {_ex_len!r}"
assert _ex_usize == 1, f"extra file_size = {_ex_usize!r}"
assert _ex_csize == 1, f"extra compress_size = {_ex_csize!r}"
assert _cd_sig == b"PK\x01\x02", f"central dir sig = {_cd_sig!r}"

# Re-reading the archive reports a single member needing ZIP64.
with zipfile.ZipFile(io.BytesIO(_zipdata)) as _z:
    _infos = _z.infolist()
    assert len(_infos) == 1, f"infolist len = {len(_infos)!r}"
    assert _infos[0].extract_version >= zipfile.ZIP64_VERSION, \
        f"extract_version = {_infos[0].extract_version!r}"

print("force_zip64_layout OK")
