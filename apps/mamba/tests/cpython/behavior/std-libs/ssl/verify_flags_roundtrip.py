# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "verify_flags_roundtrip"
# subject = "ssl.SSLContext.verify_flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_flags: verify_flags round-trips a single flag (VERIFY_CRL_CHECK_LEAF) and an OR-combined pair (| VERIFY_X509_STRICT)"""
import ssl

_vf = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
_vf.verify_flags = ssl.VERIFY_CRL_CHECK_LEAF
assert _vf.verify_flags == ssl.VERIFY_CRL_CHECK_LEAF, "single flag"
_combo = ssl.VERIFY_CRL_CHECK_LEAF | ssl.VERIFY_X509_STRICT
_vf.verify_flags = _combo
assert _vf.verify_flags == _combo, "combined flags"

print("verify_flags_roundtrip OK")
