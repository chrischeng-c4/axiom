# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "file_digest_streams_file"
# subject = "hashlib.file_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.file_digest: file_digest streams a temp file's bytes and matches the in-memory digest: by algorithm name, by constructor callable, for md5, and for an empty file"""
import hashlib

import os
import tempfile

_payload = b"hello world\n" * 100
_expected = hashlib.sha256(_payload).hexdigest()

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "data.bin")
    with open(_path, "wb") as _w:
        _w.write(_payload)

    with open(_path, "rb") as _r:
        _by_name = hashlib.file_digest(_r, "sha256").hexdigest()
    assert _by_name == _expected, f"file_digest by name = {_by_name!r}"

    with open(_path, "rb") as _r:
        _by_ctor = hashlib.file_digest(_r, lambda: hashlib.sha256()).hexdigest()
    assert _by_ctor == _expected, "file_digest by callable matches"

    with open(_path, "rb") as _r:
        _md5 = hashlib.file_digest(_r, "md5").hexdigest()
    assert _md5 == hashlib.md5(_payload).hexdigest(), "file_digest md5 matches"

    _empty_path = os.path.join(_d, "empty.bin")
    with open(_empty_path, "wb"):
        pass
    with open(_empty_path, "rb") as _r:
        _empty = hashlib.file_digest(_r, "sha256").hexdigest()
    assert _empty == hashlib.sha256(b"").hexdigest(), "file_digest of empty file"

assert not os.path.exists(_d), "tempdir auto-cleaned"

print("file_digest_streams_file OK")
