# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__read_until__match_as_bytes_wrong"
# subject = "telnetlib.Telnet.read_until(match: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed match"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed match
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.read_until(match: bytes); call it with the wrong type.

typeshed contract: match is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.read_until(12345)  # match: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
