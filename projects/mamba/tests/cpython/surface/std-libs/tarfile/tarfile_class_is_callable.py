# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "tarfile_class_is_callable"
# subject = "tarfile.TarFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.TarFile: tarfile_class_is_callable (surface)."""
import tarfile

assert callable(tarfile.TarFile)
print("tarfile_class_is_callable OK")
