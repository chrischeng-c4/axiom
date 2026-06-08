# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_truncate_before_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: truncate(n) before rollover trims the in-memory buffer in place (still not _rolled)"""
import tempfile

s = tempfile.SpooledTemporaryFile(max_size=10)
s.write(b"abcdefg\n")
s.truncate(4)
assert not s._rolled, "truncate within max_size stays in memory"
assert s._file.getvalue() == b"abcd", f"truncated buffer = {s._file.getvalue()!r}"
s.close()
print("spooled_truncate_before_rollover OK")
