# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "create_pax_header_magic_prefix"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: create_pax_header emits a bytes block beginning with the PAX extended-header magic name './@PaxHeader'"""
import tarfile

_ti = tarfile.TarInfo("foo")
_ti.mtime = 1000.1
_ti.size = 100
_ti.uid = 123
_ti.gid = 124
_info = _ti.get_info()

_hdr = _ti.create_pax_header(_info, encoding="iso8859-1")
assert isinstance(_hdr, (bytes, bytearray)), f"header type = {type(_hdr)!r}"
assert _hdr.startswith(b"././@PaxHeader"), f"header prefix = {_hdr[:14]!r}"

print("create_pax_header_magic_prefix OK")
