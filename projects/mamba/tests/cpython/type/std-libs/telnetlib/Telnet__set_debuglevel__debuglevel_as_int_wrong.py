# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__set_debuglevel__debuglevel_as_int_wrong"
# subject = "telnetlib.Telnet.set_debuglevel(debuglevel: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.set_debuglevel(debuglevel: int); call it with the wrong type.

typeshed contract: debuglevel is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.set_debuglevel("not_an_int")  # debuglevel: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
