# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "error_class_hierarchy"
# subject = "ssl.SSLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLError: the error taxonomy is nested: SSLError subclasses OSError, and SSLCertVerificationError / SSLEOFError / SSLZeroReturnError all subclass SSLError"""
import ssl

assert issubclass(ssl.SSLError, OSError), "SSLError < OSError"
assert issubclass(ssl.SSLCertVerificationError, ssl.SSLError), "CertVerify < SSLError"
assert issubclass(ssl.SSLEOFError, ssl.SSLError), "EOF < SSLError"
assert issubclass(ssl.SSLZeroReturnError, ssl.SSLError), "ZeroReturn < SSLError"

print("error_class_hierarchy OK")
