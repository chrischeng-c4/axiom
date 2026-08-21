# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "set_ecdh_curve_invalid_raises"
# subject = "ssl.SSLContext.set_ecdh_curve"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_ecdh_curve: set_ecdh_curve accepts a known curve name as str or bytes, rejects None with TypeError, and rejects an unknown curve name with ValueError"""
import ssl

_ec = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
# Known curve accepted as str or bytes.
_ec.set_ecdh_curve("prime256v1")
_ec.set_ecdh_curve(b"prime256v1")
# None -> TypeError.
try:
    _ec.set_ecdh_curve(None)
    raise AssertionError("set_ecdh_curve(None) should raise")
except TypeError:
    pass
# Unknown curve name -> ValueError.
for _bad in ("foo", b"foo"):
    try:
        _ec.set_ecdh_curve(_bad)
        raise AssertionError(f"set_ecdh_curve({_bad!r}) should raise")
    except ValueError:
        pass

print("set_ecdh_curve_invalid_raises OK")
