# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "getnames_getmembers_shapes"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: getnames returns a list of name strings (including a subdir path) and getmembers returns a matching list of TarInfo objects; getmember/extractfile resolve a member by name and TarInfo carries name/size/mode/mtime/type"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _c1 = b"Hello, tar world!"
    _i1 = tarfile.TarInfo(name="hello.txt")
    _i1.size = len(_c1)
    _tf.addfile(_i1, io.BytesIO(_c1))
    _c2 = b"Second file content here."
    _i2 = tarfile.TarInfo(name="subdir/second.txt")
    _i2.size = len(_c2)
    _tf.addfile(_i2, io.BytesIO(_c2))

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _names = _tf.getnames()
    assert isinstance(_names, list), f"getnames type = {type(_names)!r}"
    assert "hello.txt" in _names, f"hello.txt in names = {_names!r}"
    assert "subdir/second.txt" in _names, "subdir/second.txt in names"

    _members = _tf.getmembers()
    assert isinstance(_members, list), f"getmembers type = {type(_members)!r}"
    assert len(_members) == 2, f"two members = {len(_members)!r}"

    _ti = _tf.getmember("hello.txt")
    assert isinstance(_ti, tarfile.TarInfo), f"getmember type = {type(_ti)!r}"
    assert hasattr(_ti, "name"), "TarInfo has name"
    assert hasattr(_ti, "size"), "TarInfo has size"
    assert hasattr(_ti, "mode"), "TarInfo has mode"
    assert hasattr(_ti, "mtime"), "TarInfo has mtime"
    assert hasattr(_ti, "type"), "TarInfo has type"
    assert _ti.name == "hello.txt", f"name = {_ti.name!r}"
    assert _ti.size == 17, f"size = {_ti.size!r}"

    _fh = _tf.extractfile("hello.txt")
    assert _fh is not None, "extractfile not None"
    assert _fh.read() == b"Hello, tar world!", "extractfile data"

print("getnames_getmembers_shapes OK")
