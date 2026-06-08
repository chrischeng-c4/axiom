# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters__crossinterp"
# dimension = "type"
# case = "serialize_unbound__unbound_as__AnyUnbound_wrong"
# subject = "concurrent.interpreters._crossinterp.serialize_unbound(unbound: _AnyUnbound)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed unbound"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters/_crossinterp.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed unbound
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters._crossinterp.serialize_unbound(unbound: _AnyUnbound); call it with the wrong type.

typeshed contract: unbound is _AnyUnbound. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters._crossinterp import serialize_unbound
try:
    serialize_unbound(_W())  # unbound: _AnyUnbound <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
