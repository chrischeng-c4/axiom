# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "netrc"
# dimension = "type"
# case = "netrc__authenticators__host_as_str_wrong"
# subject = "netrc.netrc.authenticators(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/netrc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: netrc.netrc.authenticators(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from netrc import netrc
obj = object.__new__(netrc)
try:
    obj.authenticators(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
