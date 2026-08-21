# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "bad_cipher_string_raises"
# subject = "ssl.SSLContext.set_ciphers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_ciphers: bad_cipher_string_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().set_ciphers('no_such_cipher_suite_xyzzy')
except ssl.SSLError:
    _raised = True
assert _raised, "bad_cipher_string_raises: expected ssl.SSLError"
print("bad_cipher_string_raises OK")
