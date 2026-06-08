# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "temporary_file_is_callable"
# subject = "tempfile.TemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.TemporaryFile: temporary_file_is_callable (surface)."""
import tempfile

assert callable(tempfile.TemporaryFile)
print("temporary_file_is_callable OK")
