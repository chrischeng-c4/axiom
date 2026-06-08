# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters__crossinterp"
# dimension = "type"
# case = "resolve_unbound__flag_as__UnboundOp_wrong"
# subject = "concurrent.interpreters._crossinterp.resolve_unbound(flag: _UnboundOp)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters/_crossinterp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters._crossinterp.resolve_unbound(flag: _UnboundOp); call it with the wrong type.

typeshed contract: flag is _UnboundOp. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters._crossinterp import resolve_unbound
try:
    resolve_unbound(_W(), None)  # flag: _UnboundOp <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
