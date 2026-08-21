# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "temporary_directory_is_callable"
# subject = "tempfile.TemporaryDirectory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: temporary_directory_is_callable (surface)."""
import tempfile

assert callable(tempfile.TemporaryDirectory)
print("temporary_directory_is_callable OK")
