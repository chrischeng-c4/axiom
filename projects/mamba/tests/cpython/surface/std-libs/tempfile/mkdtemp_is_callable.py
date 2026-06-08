# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "mkdtemp_is_callable"
# subject = "tempfile.mkdtemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.mkdtemp: mkdtemp_is_callable (surface)."""
import tempfile

assert callable(tempfile.mkdtemp)
print("mkdtemp_is_callable OK")
