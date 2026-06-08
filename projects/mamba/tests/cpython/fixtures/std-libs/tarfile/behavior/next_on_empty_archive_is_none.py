# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "next_on_empty_archive_is_none"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: next() returns None at the end of an empty archive in both seekable ('r') and streaming ('r|') read modes"""
import tarfile
import io

_buf = io.BytesIO()
tarfile.open(fileobj=_buf, mode="w").close()

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r|") as _tf:
    assert _tf.next() is None, "stream next on empty"

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    assert _tf.next() is None, "seek next on empty"

print("next_on_empty_archive_is_none OK")
