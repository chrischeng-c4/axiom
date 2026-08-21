# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "mkstemp_is_callable"
# subject = "tempfile.mkstemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp_is_callable (surface)."""
import tempfile

assert callable(tempfile.mkstemp)
print("mkstemp_is_callable OK")
