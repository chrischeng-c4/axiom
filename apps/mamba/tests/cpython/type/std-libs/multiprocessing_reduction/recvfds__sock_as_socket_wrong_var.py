# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_reduction"
# dimension = "type"
# case = "recvfds__sock_as_socket_wrong_var"
# subject = "multiprocessing.reduction.recvfds(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/reduction.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.reduction.recvfds(sock: socket); call it with the
wrong type flowing through a VARIABLE rather than a direct constructor call.

Same wrong-typed-arg contract as recvfds__sock_as_socket_wrong (the direct-call
twin), but the bare instance is bound to a name first (`w = _W()`) and the name
is passed at the call site. The ① type-wall enforcement hook must reject a
walled param fed the *inferred* `Ty::Class` of a bare user class, not only the
syntactic shape of a `_W()` constructor call at the argument position (#885).

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.reduction import recvfds
try:
    w = _W()
    recvfds(w, 0)  # sock: socket <- wrong-typed, via variable
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
