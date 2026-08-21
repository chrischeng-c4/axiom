# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "spooled_negative_max_size_no_raise"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: SpooledTemporaryFile(max_size=-1) does NOT raise; a negative max_size just rolls the spool to disk on first write"""
import tempfile

spool = tempfile.SpooledTemporaryFile(max_size=-1)
spool.write(b"x" * 100)
assert spool._rolled is True, "negative max_size rolls to disk on first write"
spool.close()
print("spooled_negative_max_size_no_raise OK")
