# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "get_info_exposes_metadata"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: TarInfo.get_info() returns the header-building metadata dict carrying name/size/uid/gid and a fractional mtime preserved verbatim"""
import tarfile

_ti = tarfile.TarInfo("foo")
_ti.mtime = 1000.1
_ti.size = 100
_ti.uid = 123
_ti.gid = 124
_info = _ti.get_info()
assert _info["name"] == "foo", f"info name = {_info['name']!r}"
assert _info["size"] == 100, f"info size = {_info['size']!r}"
assert _info["uid"] == 123, f"info uid = {_info['uid']!r}"
assert _info["gid"] == 124, f"info gid = {_info['gid']!r}"
# mtime keeps its fractional part in the info dict.
assert _info["mtime"] == 1000.1, f"info mtime = {_info['mtime']!r}"

print("get_info_exposes_metadata OK")
