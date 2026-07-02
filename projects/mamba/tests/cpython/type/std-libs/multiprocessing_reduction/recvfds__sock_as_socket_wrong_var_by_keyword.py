# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_reduction"
# dimension = "type"
# case = "recvfds__sock_as_socket_wrong_var_by_keyword"
# subject = "multiprocessing.reduction.recvfds(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/reduction.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.reduction.recvfds(sock: socket); call it with the
wrong type flowing through a VARIABLE, passed BY KEYWORD.

Combines the two ① hook gaps: the bare instance is bound to a name first
(`w = _W()`, so only the inferred `Ty::Class` — not the call-expression shape —
identifies it as a bare user class; #885) and it is then passed as `sock=w`
rather than positionally, so the hook must also align the keyword arg to its
like-named `ParamSig` (#881) before running the same check.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.reduction import recvfds
try:
    w = _W()
    recvfds(sock=w, size=0)  # sock: socket <- wrong-typed, via variable, by keyword
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
