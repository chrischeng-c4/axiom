# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_capability_flags"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: readable/writable flags reflect the open mode and write methods reject a read-only BZ2File"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    assert (f.readable(), f.writable()) == (False, True), "write-mode flags"
assert f.closed is True, "closed after context exit"
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert (f.readable(), f.writable()) == (True, False), "read-mode flags"
    for op in (lambda: f.write(b"x"), lambda: f.writelines([b"x"])):
        try:
            op()
            raise AssertionError("expected OSError writing read-only file")
        except OSError:
            pass
print("bz2file_capability_flags OK")
