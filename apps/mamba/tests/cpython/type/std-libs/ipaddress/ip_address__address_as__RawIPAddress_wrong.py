# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "ip_address__address_as__RawIPAddress_wrong"
# subject = "ipaddress.ip_address(address: _RawIPAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.ip_address(address: _RawIPAddress); call it with the wrong type.

typeshed contract: address is _RawIPAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import ip_address
try:
    ip_address(_W())  # address: _RawIPAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
