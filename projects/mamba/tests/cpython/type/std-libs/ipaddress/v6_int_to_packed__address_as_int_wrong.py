# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "v6_int_to_packed__address_as_int_wrong"
# subject = "ipaddress.v6_int_to_packed(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.v6_int_to_packed(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import v6_int_to_packed
try:
    v6_int_to_packed("not_an_int")  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
