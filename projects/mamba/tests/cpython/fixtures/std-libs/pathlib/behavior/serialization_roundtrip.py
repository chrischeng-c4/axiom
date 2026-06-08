# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "serialization_roundtrip"
# subject = "pathlib.PurePath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: Path serialization: pickle round-trips preserve class/value/hash/str across protocols, bytes()/os.fspath give the OS-encoded string, as_posix always uses forward slashes, as_uri percent-encodes absolute POSIX paths, and equality against foreign types is False without raising"""
import pathlib

import os
import pickle
PurePath = pathlib.PurePath
PurePosixPath = pathlib.PurePosixPath
PosixPath = pathlib.PosixPath

# Pickle round-trips preserve class, value, hash, and string form.
_p = PurePath("/a/b")
for _proto in range(0, pickle.HIGHEST_PROTOCOL + 1):
    _back = pickle.loads(pickle.dumps(_p, _proto))
    assert _back.__class__ is _p.__class__, f"class @ proto {_proto}"
    assert _back == _p, f"value @ proto {_proto}"
    assert hash(_back) == hash(_p), f"hash @ proto {_proto}"
    assert str(_back) == str(_p), f"str @ proto {_proto}"

# bytes() and os.fspath give the OS-encoded path string.
_sep = os.fsencode(os.sep)
assert bytes(PurePath("a/b")) == b"a" + _sep + b"b", "bytes() encoding"
assert os.fspath(PurePath("a/b")) == os.path.join("a", "b"), "os.fspath"

# as_posix always uses forward slashes.
for _s in ("a", "a/b", "/", "/a/b/c"):
    assert PurePath(_s).as_posix() == _s, f"as_posix {_s!r}"

# Absolute POSIX paths produce file:// URIs, percent-encoding special bytes.
assert PosixPath("/").as_uri() == "file:///", "root uri"
assert PosixPath("/a/b.c").as_uri() == "file:///a/b.c", "plain uri"
assert PosixPath("/a/b%#c").as_uri() == "file:///a/b%25%23c", "encoded uri"

# Equality against unrelated types is always False (never raises).
assert PurePath() != "", "path != str"
assert PurePath() != {}, "path != dict"
assert PurePath() != int, "path != type"
assert PurePath("a/b") == PurePath("a", "b"), "value equality across constructors"
print("serialization_roundtrip OK")
