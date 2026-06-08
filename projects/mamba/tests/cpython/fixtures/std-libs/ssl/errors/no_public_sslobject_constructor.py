# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "no_public_sslobject_constructor"
# subject = "ssl.SSLObject"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLObject: SSLObject has no public constructor: ssl.SSLObject(MemoryBIO(), MemoryBIO()) raises TypeError naming the missing public constructor; the type is built only via wrap_bio"""
import ssl

try:
    ssl.SSLObject(ssl.MemoryBIO(), ssl.MemoryBIO())
    raise AssertionError("SSLObject() should raise")
except TypeError as _e:
    assert "public constructor" in str(_e), f"SSLObject msg: {_e}"

print("no_public_sslobject_constructor OK")
