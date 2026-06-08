# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "mktemp_is_callable"
# subject = "tempfile.mktemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.mktemp: mktemp_is_callable (surface)."""
import tempfile

assert callable(tempfile.mktemp)
print("mktemp_is_callable OK")
