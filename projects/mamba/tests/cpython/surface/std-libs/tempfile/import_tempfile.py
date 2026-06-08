# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "import_tempfile"
# subject = "tempfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile: import_tempfile (surface)."""
import tempfile

assert hasattr(tempfile, "gettempdir")
print("import_tempfile OK")
