# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "is_tarfile_is_callable"
# subject = "tarfile.is_tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.is_tarfile: is_tarfile_is_callable (surface)."""
import tarfile

assert callable(tarfile.is_tarfile)
print("is_tarfile_is_callable OK")
