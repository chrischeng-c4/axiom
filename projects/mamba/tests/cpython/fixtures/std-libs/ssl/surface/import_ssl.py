# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "import_ssl"
# subject = "ssl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl: import_ssl (surface)."""
import ssl

assert hasattr(ssl, "SSLContext")
print("import_ssl OK")
