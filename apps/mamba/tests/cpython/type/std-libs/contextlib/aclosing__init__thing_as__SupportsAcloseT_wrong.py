# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "aclosing__init__thing_as__SupportsAcloseT_wrong"
# subject = "contextlib.aclosing.__init__(thing: _SupportsAcloseT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.aclosing.__init__(thing: _SupportsAcloseT); call it with the wrong type.

typeshed contract: thing is _SupportsAcloseT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import aclosing
try:
    aclosing(_W())  # thing: _SupportsAcloseT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
