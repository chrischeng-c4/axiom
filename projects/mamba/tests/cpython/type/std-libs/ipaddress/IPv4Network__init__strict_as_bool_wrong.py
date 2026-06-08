# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "IPv4Network__init__strict_as_bool_wrong"
# subject = "ipaddress.IPv4Network.__init__(strict: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed strict"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed strict
# mamba-strict-type: TypeError
"""Type wall: ipaddress.IPv4Network.__init__(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import IPv4Network
try:
    IPv4Network(None, "not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
