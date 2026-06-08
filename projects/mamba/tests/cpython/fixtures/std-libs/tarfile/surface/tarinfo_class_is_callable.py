# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "tarinfo_class_is_callable"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.TarInfo: tarinfo_class_is_callable (surface)."""
import tarfile

assert callable(tarfile.TarInfo)
print("tarinfo_class_is_callable OK")
