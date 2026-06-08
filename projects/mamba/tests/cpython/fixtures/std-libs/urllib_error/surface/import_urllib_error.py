# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "surface"
# case = "import_urllib_error"
# subject = "urllib.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error: import_urllib_error (surface)."""
import urllib.error

assert hasattr(urllib.error, "URLError")
print("import_urllib_error OK")
