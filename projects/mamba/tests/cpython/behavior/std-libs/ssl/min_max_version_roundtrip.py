# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "min_max_version_roundtrip"
# subject = "ssl.SSLContext.minimum_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.minimum_version: minimum_version / maximum_version round-trip: setting them to TLSv1_2 / TLSv1_3 reads back the same members"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
_ctx.minimum_version = ssl.TLSVersion.TLSv1_2
_ctx.maximum_version = ssl.TLSVersion.TLSv1_3
assert _ctx.minimum_version == ssl.TLSVersion.TLSv1_2, "min version set"
assert _ctx.maximum_version == ssl.TLSVersion.TLSv1_3, "max version set"

print("min_max_version_roundtrip OK")
