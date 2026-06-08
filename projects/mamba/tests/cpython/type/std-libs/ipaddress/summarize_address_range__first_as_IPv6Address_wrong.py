# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "summarize_address_range__first_as_IPv6Address_wrong"
# subject = "ipaddress.summarize_address_range(first: IPv6Address)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first
# mamba-strict-type: TypeError
"""Type wall: ipaddress.summarize_address_range(first: IPv6Address); call it with the wrong type.

typeshed contract: first is IPv6Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import summarize_address_range
try:
    summarize_address_range(_W(), None)  # first: IPv6Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
