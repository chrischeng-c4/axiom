# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "open_is_callable"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: open_is_callable (surface)."""
import tarfile

assert callable(tarfile.open)
print("open_is_callable OK")
