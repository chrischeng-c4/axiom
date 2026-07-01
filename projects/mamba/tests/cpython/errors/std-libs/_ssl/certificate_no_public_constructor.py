# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "errors"
# case = "certificate_no_public_constructor"
# subject = "_ssl.Certificate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "CPython 3.12 _ssl module"
# status = "filled"
# ///
"""_ssl.Certificate: public construction is rejected with CPython safety errors."""
from _ssl import Certificate

try:
    Certificate()
    raise AssertionError("Certificate() should raise")
except TypeError as _e:
    assert "cannot create '_ssl.Certificate' instances" in str(_e), str(_e)

try:
    object.__new__(Certificate)
    raise AssertionError("object.__new__(Certificate) should raise")
except TypeError as _e:
    assert "object.__new__(_ssl.Certificate) is not safe" in str(_e), str(_e)

print("certificate_no_public_constructor OK")
