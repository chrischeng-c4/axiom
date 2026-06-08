# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "DER_cert_to_PEM_cert__der_cert_bytes_as_ReadableBuffer_wrong"
# subject = "ssl.DER_cert_to_PEM_cert(der_cert_bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.DER_cert_to_PEM_cert(der_cert_bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: der_cert_bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import DER_cert_to_PEM_cert
try:
    DER_cert_to_PEM_cert(_W())  # der_cert_bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
