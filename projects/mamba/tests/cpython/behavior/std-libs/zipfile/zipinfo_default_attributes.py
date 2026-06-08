# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_default_attributes"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: a bare ZipInfo() carries the documented defaults: NoName filename, date_time (1980,1,1,0,0,0), ZIP_STORED compress_type, empty comment/extra, DEFAULT_VERSION create/extract version, and zero sizes/flags"""
import zipfile

_zi = zipfile.ZipInfo()
assert _zi.orig_filename == "NoName", f"orig_filename = {_zi.orig_filename!r}"
assert _zi.filename == "NoName", f"filename = {_zi.filename!r}"
assert _zi.date_time == (1980, 1, 1, 0, 0, 0), f"date_time = {_zi.date_time!r}"
assert _zi.compress_type == zipfile.ZIP_STORED, f"compress_type = {_zi.compress_type!r}"
assert _zi.comment == b"", f"comment = {_zi.comment!r}"
assert _zi.extra == b"", f"extra = {_zi.extra!r}"
assert _zi.create_system in (0, 3), f"create_system = {_zi.create_system!r}"
assert _zi.create_version == zipfile.DEFAULT_VERSION, f"create_version = {_zi.create_version!r}"
assert _zi.extract_version == zipfile.DEFAULT_VERSION, f"extract_version = {_zi.extract_version!r}"
assert _zi.reserved == 0, f"reserved = {_zi.reserved!r}"
assert _zi.flag_bits == 0, f"flag_bits = {_zi.flag_bits!r}"
assert _zi.volume == 0, f"volume = {_zi.volume!r}"
assert _zi.internal_attr == 0, f"internal_attr = {_zi.internal_attr!r}"
assert _zi.external_attr == 0, f"external_attr = {_zi.external_attr!r}"
assert _zi.file_size == 0, f"file_size = {_zi.file_size!r}"
assert _zi.compress_size == 0, f"compress_size = {_zi.compress_size!r}"

print("zipinfo_default_attributes OK")
