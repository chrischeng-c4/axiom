# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_servername_callback__server_name_callback_as_typed_wrong"
# subject = "ssl.SSLContext.set_servername_callback(server_name_callback: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_servername_callback(server_name_callback: typed); call it with the wrong type.

typeshed contract: server_name_callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_servername_callback(_W())  # server_name_callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
