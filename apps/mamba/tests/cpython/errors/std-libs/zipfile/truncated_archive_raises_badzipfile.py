# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "truncated_archive_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: every proper prefix of a valid archive (truncated to n bytes for all n) fails to open with BadZipFile"""
import zipfile
import io

_good = io.BytesIO()
with zipfile.ZipFile(_good, "w") as _zf:
    _zf.writestr("foo.txt", b"O, for a Muse of Fire!")
_blob = _good.getvalue()

for _n in range(len(_blob)):
    _raised = False
    try:
        zipfile.ZipFile(io.BytesIO(_blob[:_n]))
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, f"truncated to {_n} bytes -> BadZipFile"

print("truncated_archive_raises_badzipfile OK")
