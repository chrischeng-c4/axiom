# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "wrap_socket__sock_as_socket_wrong"
# subject = "ssl.wrap_socket(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.wrap_socket(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import wrap_socket
try:
    wrap_socket(_W())  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
