# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_peek_readinto"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File peek returns a leading prefix and readinto fills a bytearray and advances the position"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    peek = f.peek()
    assert len(peek) != 0 and b"0123456789abcdef".startswith(peek), "peek prefix"
    ba = bytearray(8)
    assert f.readinto(ba) == 8, "readinto count"
    assert bytes(ba) == b"01234567", f"readinto bytes = {bytes(ba)!r}"
    assert f.read() == b"89abcdef", "read remainder"
print("bz2file_peek_readinto OK")
