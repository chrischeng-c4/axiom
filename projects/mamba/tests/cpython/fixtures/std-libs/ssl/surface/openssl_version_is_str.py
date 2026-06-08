# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "openssl_version_is_str"
# subject = "ssl.OPENSSL_VERSION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.OPENSSL_VERSION: openssl_version_is_str (surface)."""
import ssl

assert type(ssl.OPENSSL_VERSION).__name__ == "str"
print("openssl_version_is_str OK")
