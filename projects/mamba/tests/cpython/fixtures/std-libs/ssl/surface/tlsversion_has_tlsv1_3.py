# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "tlsversion_has_tlsv1_3"
# subject = "ssl.TLSVersion"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.TLSVersion: tlsversion_has_tlsv1_3 (surface)."""
import ssl

assert hasattr(ssl.TLSVersion, "TLSv1_3")
print("tlsversion_has_tlsv1_3 OK")
