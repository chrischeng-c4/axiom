# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_socket"
# dimension = "type"
# case = "inet_pton__address_family_as_int_wrong"
# subject = "_socket.inet_pton(address_family: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_socket.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _socket.inet_pton(address_family: int); call it with the wrong type.

typeshed contract: address_family is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _socket import inet_pton
try:
    inet_pton("not_an_int", "")  # address_family: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
