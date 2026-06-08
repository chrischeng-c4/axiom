# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "PEM_cert_to_DER_cert__pem_cert_string_as_str_wrong"
# subject = "ssl.PEM_cert_to_DER_cert(pem_cert_string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.PEM_cert_to_DER_cert(pem_cert_string: str); call it with the wrong type.

typeshed contract: pem_cert_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import PEM_cert_to_DER_cert
try:
    PEM_cert_to_DER_cert(12345)  # pem_cert_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
