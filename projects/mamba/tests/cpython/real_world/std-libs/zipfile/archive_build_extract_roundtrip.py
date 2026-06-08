# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "real_world"
# case = "archive_build_extract_roundtrip"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: an end-user packaging flow: build a multi-member DEFLATED archive on disk (text + nested binary), confirm is_zipfile detects it, reopen and read each member back, then extractall into a separate temp directory and verify every extracted file's bytes match the originals"""
import zipfile
import os
import tempfile

# The payload an end-user wants to package: a README, a config under a
# subdir, and a small binary asset (the full 0..255 byte range).
_members = {
    "README.txt": b"Project bundle\nversion 1\n",
    "conf/settings.ini": b"[server]\nport = 8080\n",
    "assets/icon.bin": bytes(range(256)) * 4,
}

with tempfile.TemporaryDirectory() as _td:
    _archive = os.path.join(_td, "bundle.zip")

    # Build the archive with DEFLATE compression.
    with zipfile.ZipFile(_archive, "w", compression=zipfile.ZIP_DEFLATED) as _zf:
        for _name, _content in _members.items():
            _zf.writestr(_name, _content)

    # is_zipfile recognises the freshly written archive.
    assert zipfile.is_zipfile(_archive), "is_zipfile must detect the bundle"

    # Reopen and read every member back in place.
    with zipfile.ZipFile(_archive, "r") as _zf:
        assert sorted(_zf.namelist()) == sorted(_members), \
            f"namelist = {_zf.namelist()!r}"
        for _name, _content in _members.items():
            assert _zf.read(_name) == _content, f"read mismatch: {_name}"

    # Extract everything into a separate directory and verify on disk.
    _dest = os.path.join(_td, "unpacked")
    with zipfile.ZipFile(_archive, "r") as _zf:
        _zf.extractall(_dest)
    for _name, _content in _members.items():
        _path = os.path.join(_dest, *_name.split("/"))
        assert os.path.exists(_path), f"missing extracted file: {_name}"
        with open(_path, "rb") as _f:
            assert _f.read() == _content, f"extracted content mismatch: {_name}"

print("archive_build_extract_roundtrip OK")
