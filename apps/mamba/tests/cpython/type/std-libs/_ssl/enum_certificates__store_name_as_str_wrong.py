# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "enum_certificates__store_name_as_str_wrong"
# subject = "_ssl.enum_certificates(store_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.enum_certificates(store_name: str); call it with the wrong type.

typeshed contract: store_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import enum_certificates
try:
    enum_certificates(12345)  # store_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
