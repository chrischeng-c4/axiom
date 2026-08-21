# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncore"
# dimension = "type"
# case = "dispatcher__set_socket__sock_as__Socket_wrong"
# subject = "asyncore.dispatcher.set_socket(sock: _Socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncore.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncore.dispatcher.set_socket(sock: _Socket); call it with the wrong type.

typeshed contract: sock is _Socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncore import dispatcher
obj = object.__new__(dispatcher)
try:
    obj.set_socket(_W())  # sock: _Socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
