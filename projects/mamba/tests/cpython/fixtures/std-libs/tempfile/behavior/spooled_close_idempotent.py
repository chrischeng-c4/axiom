# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_close_idempotent"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: close() on a SpooledTemporaryFile is idempotent both before and after rollover"""
import tempfile

for size, label in [(1024, "before"), (1, "after")]:
    c = tempfile.SpooledTemporaryFile(max_size=size)
    c.write(b"abc\n")
    c.close()
    c.close()
    c.close()
print("spooled_close_idempotent OK")
