# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "import_base64"
# subject = "base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64: import_base64 (surface)."""
import base64

assert hasattr(base64, "b64encode")
print("import_base64 OK")
