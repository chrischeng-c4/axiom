# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_close_idempotent"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: calling close() repeatedly on a NamedTemporaryFile is idempotent (no error on the 2nd/3rd call)"""
import tempfile

g = tempfile.NamedTemporaryFile()
g.write(b"abc\n")
g.close()
g.close()
g.close()
print("named_file_close_idempotent OK")
