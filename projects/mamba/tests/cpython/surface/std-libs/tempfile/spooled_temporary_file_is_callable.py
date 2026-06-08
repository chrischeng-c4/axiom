# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "spooled_temporary_file_is_callable"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: spooled_temporary_file_is_callable (surface)."""
import tempfile

assert callable(tempfile.SpooledTemporaryFile)
print("spooled_temporary_file_is_callable OK")
