# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "named_temporary_file_is_callable"
# subject = "tempfile.NamedTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: named_temporary_file_is_callable (surface)."""
import tempfile

assert callable(tempfile.NamedTemporaryFile)
print("named_temporary_file_is_callable OK")
