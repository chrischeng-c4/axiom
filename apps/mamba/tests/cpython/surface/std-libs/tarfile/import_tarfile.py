# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "import_tarfile"
# subject = "tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile: import_tarfile (surface)."""
import tarfile

assert hasattr(tarfile, "open")
print("import_tarfile OK")
