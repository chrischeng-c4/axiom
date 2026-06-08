# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "multiple_members_independent"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: several members written to one archive are each retrieved independently with their own content"""
import zipfile
import io

_files = {"a.txt": b"alpha", "b.txt": b"beta", "c.txt": b"gamma"}
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    for _name, _content in _files.items():
        _zf.writestr(_name, _content)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    for _name, _content in _files.items():
        assert _zf.read(_name) == _content, f"content mismatch: {_name}"

print("multiple_members_independent OK")
