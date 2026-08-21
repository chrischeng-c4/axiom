# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "cert_time_to_seconds__cert_time_as_str_wrong"
# subject = "ssl.cert_time_to_seconds(cert_time: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.cert_time_to_seconds(cert_time: str); call it with the wrong type.

typeshed contract: cert_time is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import cert_time_to_seconds
try:
    cert_time_to_seconds(12345)  # cert_time: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
