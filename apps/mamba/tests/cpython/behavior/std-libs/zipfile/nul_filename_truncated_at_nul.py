# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "nul_filename_truncated_at_nul"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: a member name containing a NUL byte is truncated at the NUL so namelist reports the prefix only"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt\x00qqq", b"O, for a Muse of Fire!")
    assert _zf.namelist() == ["foo.txt"], f"NUL-truncated names = {_zf.namelist()!r}"

print("nul_filename_truncated_at_nul OK")
