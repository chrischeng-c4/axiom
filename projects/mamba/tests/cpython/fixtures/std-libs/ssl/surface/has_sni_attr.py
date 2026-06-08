# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "has_sni_attr"
# subject = "ssl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl: has_sni_attr (surface)."""
import ssl

assert hasattr(ssl, "HAS_SNI")
print("has_sni_attr OK")
