# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__set_option_negotiation_callback__callback_as_typed_wrong"
# subject = "telnetlib.Telnet.set_option_negotiation_callback(callback: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callback"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callback
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.set_option_negotiation_callback(callback: typed); call it with the wrong type.

typeshed contract: callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.set_option_negotiation_callback(_W())  # callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
