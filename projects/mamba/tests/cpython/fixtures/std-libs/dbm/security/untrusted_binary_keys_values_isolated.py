# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "security"
# case = "untrusted_binary_keys_values_isolated"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: untrusted binary keys/values (NUL bytes, path-traversal-looking strings, long blobs) are stored and read back verbatim by the dumb backend with no interpretation or collision"""
import dbm.dumb
import os
import tempfile

# Adversarial inputs: embedded NUL bytes, a path-traversal-looking key, and a
# large opaque blob. The store must treat all of them as opaque byte strings.
_payloads = {
    b"a\x00b": b"v\x00w",                       # NUL inside both key and value
    b"../../etc/passwd": b"not-a-path",          # path-traversal-looking key
    b"\xff\xfe\x00\x01": b"\x00" * 256,          # non-UTF-8 key, all-NUL value
    b"blob": b"Z" * 50_000,                      # large opaque blob
}

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "untrusted")
    with dbm.dumb.open(_path, "c") as _db:
        for _k, _v in _payloads.items():
            _db[_k] = _v
    with dbm.dumb.open(_path, "r") as _db:
        # Every adversarial pair round-trips byte-for-byte.
        assert len(_db) == len(_payloads), f"all keys distinct = {len(_db)!r}"
        for _k, _v in _payloads.items():
            assert _db[_k] == _v, f"verbatim read for {_k!r}"
        # A NUL-containing key does NOT collide with its NUL-stripped sibling.
        assert b"ab" not in _db, "NUL key not conflated with 'ab'"

print("untrusted_binary_keys_values_isolated OK")
