# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "has_alpn_attr"
# subject = "ssl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl: has_alpn_attr (surface)."""
import ssl

assert hasattr(ssl, "HAS_ALPN")
print("has_alpn_attr OK")
